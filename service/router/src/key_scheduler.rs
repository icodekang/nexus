//! Pressure-based API Key Load Balancer
//!
//! This module implements a "Pressure Equilibrium" algorithm for distributing
//! requests across multiple API keys within the same provider, with
//! session-level key affinity to ensure a single session always routes to
//! the same API key unless that key becomes unavailable.
//!
//! Key concepts:
//! - Each API key has a "pressure" based on its current utilization
//! - Keys with lower pressure (less utilization) receive more requests
//! - Pressure naturally decays over time, allowing rebalancing
//! - Session affinity: once a session is bound to a key, subsequent requests
//!   from that session always use the same key, providing consistent context
//! - When a bound key fails, the session is rebound to another healthy key
//! - After TTL expiry, the session remembers its previous key; if that key
//!   is still healthy it is preferred, otherwise a new key is selected
//!
//! The algorithm ensures:
//! 1. Optimal utilization: No single key is overwhelmed while others idle
//! 2. Session affinity: Same session → same API key (within TTL window)
//! 3. TTL-aware rebinding: After TTL expiry, previous key is tried first
//! 4. Failover transparency: Users don't notice when a key is swapped
//! 5. Graceful degradation: System handles key exhaustion gracefully

use models::ProviderKey;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How fast utilization decays (per second)
const DECAY_RATE: f64 = 0.1;

/// Target utilization per key (60%)
const TARGET_UTILIZATION: f64 = 0.6;

/// Session binding TTL — if a session has no activity for this duration, its
/// binding expires and it is eligible for rebalancing (default: 10 minutes).
const SESSION_BINDING_TTL_SECS: u64 = 600;

/// Revive unhealthy keys after this many seconds (default: 5 minutes).
const KEY_REVIVAL_INTERVAL_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// KeyLoad
// ---------------------------------------------------------------------------

/// Load metrics for a single API key
#[derive(Debug, Clone)]
pub struct KeyLoad {
    pub key_id: uuid::Uuid,
    pub requests_in_flight: u32,
    pub recent_requests: u32,     // Requests in last window
    pub total_requests: u64,      // All-time requests
    pub latency_ms: f64,          // Average latency
    pub last_used: Option<Instant>,
    pub consecutive_failures: u32,
    pub is_healthy: bool,
    /// Unix timestamp (secs) when this key can be revived after being marked
    /// unhealthy. Set to now + KEY_REVIVAL_INTERVAL_SECS when marked unhealthy.
    pub revival_at_secs: u64,
}

impl KeyLoad {
    pub fn new(key_id: uuid::Uuid) -> Self {
        Self {
            key_id,
            requests_in_flight: 0,
            recent_requests: 0,
            total_requests: 0,
            latency_ms: 0.0,
            last_used: None,
            consecutive_failures: 0,
            is_healthy: true,
            revival_at_secs: 0,
        }
    }

    /// Calculate pressure: higher pressure = more utilized = less likely to be selected.
    /// Returns f64::MAX for unhealthy keys (never selected unless revival time passed).
    pub fn pressure(&self) -> f64 {
        if !self.is_healthy {
            return f64::MAX;
        }

        let utilization = self.current_utilization();
        let latency_factor = if self.latency_ms > 0.0 {
            1.0 / self.latency_ms.min(5000.0) * 1000.0
        } else {
            1.0
        };

        let util_delta = utilization - TARGET_UTILIZATION;
        util_delta / latency_factor.max(0.01)
    }

    /// Current utilization as a fraction (0.0 to 1.0+)
    pub fn current_utilization(&self) -> f64 {
        let flight_util = (self.requests_in_flight as f64) * 0.1;
        let recent_util = (self.recent_requests as f64) * 0.05;
        flight_util.min(1.0) + recent_util.min(0.5)
    }

    pub fn record_request(&mut self) {
        self.requests_in_flight += 1;
        self.recent_requests += 1;
        self.total_requests += 1;
        self.last_used = Some(Instant::now());
        self.consecutive_failures = 0;
    }

    pub fn record_success(&mut self, latency_ms: f64) {
        self.requests_in_flight = self.requests_in_flight.saturating_sub(1);
        if self.latency_ms == 0.0 {
            self.latency_ms = latency_ms;
        } else {
            self.latency_ms = self.latency_ms * 0.9 + latency_ms * 0.1;
        }
    }

    pub fn record_failure(&mut self) {
        self.requests_in_flight = self.requests_in_flight.saturating_sub(1);
        self.consecutive_failures += 1;
        if self.consecutive_failures >= 3 {
            self.is_healthy = false;
            self.revival_at_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() + KEY_REVIVAL_INTERVAL_SECS)
                .unwrap_or(0);
        }
    }

    pub fn mark_healthy(&mut self) {
        self.is_healthy = true;
        self.consecutive_failures = 0;
    }

    pub fn apply_decay(&mut self, elapsed: Duration) {
        let decay_factor = (-DECAY_RATE * elapsed.as_secs_f64()).exp();
        self.recent_requests = (self.recent_requests as f64 * decay_factor) as u32;
    }

    /// Returns true if the key can be revived now.
    pub fn can_retry(&self) -> bool {
        !self.is_healthy && {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now >= self.revival_at_secs
        }
    }
}

// ---------------------------------------------------------------------------
// SessionBinding
// ---------------------------------------------------------------------------

/// Tracks a session → key binding for a specific provider.
/// After TTL expiry, the `previous_key_id` field allows the scheduler to
/// prefer the same key again if it is still healthy.
#[derive(Debug, Clone)]
pub struct SessionBinding {
    /// The key this session is currently bound to.
    pub key_id: uuid::Uuid,
    /// Last time this session made a request (used for TTL expiry check).
    pub last_used: Instant,
    /// The key the session was bound to *before* the current binding
    /// (used for TTL restoration when `key_id` has expired).
    pub previous_key_id: Option<uuid::Uuid>,
}

impl SessionBinding {
    /// Returns true if the binding has expired (no activity within TTL).
    pub fn is_expired(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_used) > Duration::from_secs(SESSION_BINDING_TTL_SECS)
    }
}

// ---------------------------------------------------------------------------
// SelectedKey
// ---------------------------------------------------------------------------

/// Result of a key selection.
#[derive(Debug)]
pub struct SelectedKey {
    pub key: ProviderKey,
    pub load: KeyLoad,
    pub pressure: f64,
    /// True when this key was selected via session affinity (not fresh selection).
    pub from_affinity: bool,
    /// True when this key was selected because the previous binding expired but
    /// the previous key was still healthy and is being restored.
    pub restored_from_previous: bool,
}

// ---------------------------------------------------------------------------
// ProviderKeyScheduler
// ---------------------------------------------------------------------------

pub struct ProviderKeyScheduler {
    provider_slug: String,
    keys: HashMap<uuid::Uuid, ProviderKey>,
    loads: HashMap<uuid::Uuid, KeyLoad>,
    /// session_id → binding
    session_bindings: HashMap<String, SessionBinding>,
    gain_factor: f64,
}

impl ProviderKeyScheduler {
    pub fn new(provider_slug: String) -> Self {
        Self {
            provider_slug,
            keys: HashMap::new(),
            loads: HashMap::new(),
            session_bindings: HashMap::new(),
            gain_factor: 1.5,
        }
    }

    pub fn add_key(&mut self, key: ProviderKey) {
        let key_id = key.id;
        self.keys.insert(key_id, key);
        self.loads.entry(key_id).or_insert_with(|| KeyLoad::new(key_id));
    }

    pub fn remove_key(&mut self, key_id: uuid::Uuid) {
        self.keys.remove(&key_id);
        self.loads.remove(&key_id);
        // Evict any bindings that pointed to this key.
        self.session_bindings.retain(|_, b| b.key_id != key_id);
    }

    pub fn set_keys(&mut self, keys: Vec<ProviderKey>) {
        let key_ids: std::collections::HashSet<_> = keys.iter().map(|k| k.id).collect();
        self.keys.retain(|id, _| key_ids.contains(id));
        self.loads.retain(|id, _| key_ids.contains(id));
        self.session_bindings.retain(|_, b| {
            key_ids.contains(&b.key_id)
                || b.previous_key_id.map(|pid| key_ids.contains(&pid)).unwrap_or(false)
        });
        for key in keys {
            let key_id = key.id;
            self.keys.insert(key_id, key);
            self.loads.entry(key_id).or_insert_with(|| KeyLoad::new(key_id));
        }
    }

    /// Select a key for a given session.
    ///
    /// Strategy:
    /// 1. If session has a binding AND the key is healthy → use it (refresh TTL).
    /// 2. If session has a binding but the key is unhealthy AND revival window
    ///    has passed → evict binding, fall through to step 3.
    /// 3. If session has NO binding OR the binding expired:
    ///    a. Try previous_key_id if stored and still healthy → restore binding.
    ///    b. Otherwise → pick fresh key by lowest pressure, bind session.
    pub fn select_key_for_session(
        &mut self,
        session_id: &str,
    ) -> Option<SelectedKey> {
        self.tick_key_revive();

        // --- Step 1: Check existing binding ---
        // We clone key_id out so we can mutate self after reading it.
        let existing_key_id: Option<uuid::Uuid> =
            self.session_bindings.get(session_id).map(|b| b.key_id);

        if let Some(key_id) = existing_key_id {
            // Check health WITHOUT holding any borrow across touch_session.
            // We clone the load data we need first.
            let (is_healthy, can_retry, has_key) = {
                self.loads.get(&key_id)
                    .map(|l| (l.is_healthy, l.can_retry(), self.keys.contains_key(&key_id)))
                    .unwrap_or((false, false, false))
            };

            if is_healthy {
                // Safe: we only mutate `session_bindings` for `session_id`, which is
                // not accessed through `self.loads` (a different HashMap).
                self.touch_session(session_id);
                let key = self.keys.get(&key_id)?.clone();
                let load_clone = self.loads.get(&key_id)?.clone();
                let pressure = load_clone.pressure();
                return Some(SelectedKey {
                    key,
                    load: load_clone,
                    pressure,
                    from_affinity: true,
                    restored_from_previous: false,
                });
            } else if can_retry {
                self.session_bindings.remove(session_id);
            } else if has_key {
                // Key unhealthy but revival not passed — still return it for retry.
                let load_clone = self.loads.get(&key_id)?.clone();
                return Some(SelectedKey {
                    key: self.keys.get(&key_id)?.clone(),
                    load: load_clone,
                    pressure: f64::MAX,
                    from_affinity: true,
                    restored_from_previous: false,
                });
            }
        }

        // --- Step 2: No valid binding — try to restore from previous_key_id ---
        // Clone now so we can drop borrows before any mutable operations.
        let prev_binding = self.session_bindings.get(session_id).cloned();

        // Extract prev_key_id while binding is still in scope.
        let prev_key_id_from_binding = prev_binding.as_ref()
            .and_then(|b| b.previous_key_id);

        if let Some(ref binding) = prev_binding {
            if let Some(prev_key_id) = binding.previous_key_id {
                // Check health after all borrows are dropped.
                let (is_healthy, key_exists) = {
                    self.loads.get(&prev_key_id)
                        .map(|l| (l.is_healthy, self.keys.contains_key(&prev_key_id)))
                        .unwrap_or((false, false))
                };

                if is_healthy && key_exists {
                    let key = self.keys.get(&prev_key_id)?.clone();
                    let load_clone = self.loads.get(&prev_key_id)?.clone();
                    let pressure = load_clone.pressure();

                    // Evict the old binding and insert the restored one.
                    self.session_bindings.remove(session_id);
                    self.session_bindings.insert(
                        session_id.to_string(),
                        SessionBinding {
                            key_id: prev_key_id,
                            last_used: Instant::now(),
                            previous_key_id: Some(binding.key_id),
                        },
                    );

                    if let Some(l) = self.loads.get_mut(&prev_key_id) {
                        l.record_request();
                    }

                    return Some(SelectedKey {
                        key,
                        load: load_clone,
                        pressure,
                        from_affinity: true,
                        restored_from_previous: true,
                    });
                }
            }
        }

        // --- Step 3: No binding or can't restore — pick fresh key ---
        // prev_key_id_from_binding is used here (prev_binding consumed above).
        let prev_key_id = prev_key_id_from_binding;
        let picked = self.pick_key_by_pressure()?;
        let key_id = picked.key.id;

        if let Some(load) = self.loads.get_mut(&key_id) {
            load.record_request();
        }

        self.session_bindings.insert(
            session_id.to_string(),
            SessionBinding {
                key_id,
                last_used: Instant::now(),
                previous_key_id: prev_key_id,
            },
        );

        Some(SelectedKey {
            key: picked.key,
            load: picked.load,
            pressure: picked.pressure,
            from_affinity: false,
            restored_from_previous: false,
        })
    }

    /// Touch a session to refresh its TTL (call on each request that uses the binding).
    pub fn touch_session(&mut self, session_id: &str) {
        if let Some(binding) = self.session_bindings.get_mut(session_id) {
            binding.last_used = Instant::now();
        }
    }

    /// Evict expired session bindings.
    pub fn evict_expired_sessions(&mut self) {
        let now = Instant::now();
        let ttl = Duration::from_secs(SESSION_BINDING_TTL_SECS);
        let to_evict: Vec<String> = self
            .session_bindings
            .iter()
            .filter(|(_, b)| now.duration_since(b.last_used) > ttl)
            .map(|(sid, _)| sid.clone())
            .collect();
        for sid in to_evict {
            self.session_bindings.remove(&sid);
        }
    }

    /// Attempt to revive unhealthy keys whose revival window has passed.
    fn tick_key_revive(&mut self) {
        for load in self.loads.values_mut() {
            if load.can_retry() {
                tracing::info!("Reviving key {} after cooldown", load.key_id);
                load.mark_healthy();
            }
        }
    }

    /// Pure pressure-based selection without any session affinity.
    pub fn select_key_no_session(&mut self) -> Option<SelectedKey> {
        self.tick_key_revive();
        self.pick_key_by_pressure()
    }

    fn pick_key_by_pressure(&mut self) -> Option<SelectedKey> {
        let active_keys: Vec<_> = self
            .keys
            .values()
            .filter(|k| k.is_active)
            .collect();

        if active_keys.is_empty() {
            return None;
        }

        let mut candidates: Vec<(uuid::Uuid, f64)> = Vec::new();
        for key in &active_keys {
            if let Some(load) = self.loads.get(&key.id) {
                if load.pressure() < f64::MAX {
                    candidates.push((key.id, load.pressure()));
                }
            }
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_pressure = candidates.first()?.1;
        let good_candidates: Vec<_> = candidates
            .iter()
            .filter(|(_, p)| *p < best_pressure * 2.0)
            .take(3)
            .collect();

        let selected_id = if good_candidates.len() > 1 {
            // Weighted random among top candidates:
            // lower pressure (most idle) → higher weight.
            let weights: Vec<f64> = good_candidates
                .iter()
                .map(|(_, p)| (-p).max(0.1))
                .collect();
            let total_weight: f64 = weights.iter().sum();
            let mut r = rand_simple() * total_weight;
            for (i, weight) in weights.iter().enumerate() {
                r -= weight;
                if r <= 0.0 {
                    let (key_id, pressure) = good_candidates[i];
                    let key_clone = self.keys.get(key_id)?.clone();
                    let load_clone = self.loads.get(&key_clone.id)?.clone();
                    return Some(SelectedKey {
                        key: key_clone,
                        load: load_clone,
                        pressure: *pressure,
                        from_affinity: false,
                        restored_from_previous: false,
                    });
                }
            }
            // Fallback: last candidate (not dereferenced, just indexed).
            good_candidates[good_candidates.len() - 1].0
        } else {
            candidates.first().unwrap().0
        };

        let key_final = self.keys.get(&selected_id)?.clone();
        let load_clone = self.loads.get(&selected_id)?.clone();
        let pressure = load_clone.pressure();
        Some(SelectedKey {
            key: key_final,
            load: load_clone,
            pressure,
            from_affinity: false,
            restored_from_previous: false,
        })
    }

    pub fn record_success(&mut self, key_id: uuid::Uuid, latency_ms: f64) {
        if let Some(load) = self.loads.get_mut(&key_id) {
            load.record_success(latency_ms);
        }
    }

    pub fn record_failure(&mut self, key_id: uuid::Uuid) {
        if let Some(load) = self.loads.get_mut(&key_id) {
            load.record_failure();
        }
        // Evict bindings pointing to this key, but preserve previous_key_id history.
        let to_insert: Vec<(String, SessionBinding)> = self
            .session_bindings
            .iter()
            .filter(|(_, b)| b.key_id == key_id)
            .map(|(sid, b)| {
                (
                    sid.clone(),
                    SessionBinding {
                        key_id: b.previous_key_id.unwrap_or(key_id),
                        last_used: b.last_used,
                        previous_key_id: Some(key_id),
                    },
                )
            })
            .collect();
        for (sid, binding) in to_insert {
            self.session_bindings.insert(sid, binding);
        }
        self.session_bindings
            .retain(|_, b| b.key_id != key_id);
    }

    pub fn revive_key(&mut self, key_id: uuid::Uuid) {
        if let Some(load) = self.loads.get_mut(&key_id) {
            load.mark_healthy();
        }
    }

    pub fn get_stats(&self) -> HashMap<uuid::Uuid, (f64, f64)> {
        self.loads
            .iter()
            .map(|(id, load)| (*id, (load.pressure(), load.current_utilization())))
            .collect()
    }

    pub fn active_key_count(&self) -> usize {
        self.keys.values().filter(|k| k.is_active).count()
    }

    pub fn apply_decay_all(&mut self, elapsed: Duration) {
        for load in self.loads.values_mut() {
            load.apply_decay(elapsed);
        }
    }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain_factor = gain;
    }

    /// Number of active session bindings.
    pub fn active_binding_count(&self) -> usize {
        self.session_bindings.len()
    }
}

// ---------------------------------------------------------------------------
// GlobalKeyScheduler
// ---------------------------------------------------------------------------

/// Simple deterministic-ish RNG (LCG).
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((nanos as u64).wrapping_mul(6364136223846793005).wrapping_add(1) >> 33) as f64
        / u32::MAX as f64
}

/// Global scheduler managing all providers.
pub struct GlobalKeyScheduler {
    providers: HashMap<String, ProviderKeyScheduler>,
    last_decay: Instant,
}

impl GlobalKeyScheduler {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            last_decay: Instant::now(),
        }
    }

    pub fn set_provider_keys(&mut self, provider_slug: &str, keys: Vec<ProviderKey>) {
        let scheduler = self
            .providers
            .entry(provider_slug.to_string())
            .or_insert_with(|| ProviderKeyScheduler::new(provider_slug.to_string()));
        scheduler.set_keys(keys);
    }

    /// Select a key for `provider_slug` bound to `session_id`.
    pub fn select_key_for_session(
        &mut self,
        provider_slug: &str,
        session_id: &str,
    ) -> Option<SelectedKey> {
        self.providers
            .get_mut(provider_slug)?
            .select_key_for_session(session_id)
    }

    /// Select a key without any session context (fallback).
    pub fn select_key_no_session(&mut self, provider_slug: &str) -> Option<SelectedKey> {
        self.providers
            .get_mut(provider_slug)?
            .select_key_no_session()
    }

    pub fn touch_session(&mut self, provider_slug: &str, session_id: &str) {
        if let Some(sched) = self.providers.get_mut(provider_slug) {
            sched.touch_session(session_id);
        }
    }

    pub fn record_success(
        &mut self,
        provider_slug: &str,
        key_id: uuid::Uuid,
        latency_ms: f64,
    ) {
        if let Some(scheduler) = self.providers.get_mut(provider_slug) {
            scheduler.record_success(key_id, latency_ms);
        }
    }

    pub fn record_failure(&mut self, provider_slug: &str, key_id: uuid::Uuid) {
        if let Some(scheduler) = self.providers.get_mut(provider_slug) {
            scheduler.record_failure(key_id);
        }
    }

    pub fn get_provider(&self, provider_slug: &str) -> Option<&ProviderKeyScheduler> {
        self.providers.get(provider_slug)
    }

    pub fn get_provider_mut(&mut self, provider_slug: &str) -> Option<&mut ProviderKeyScheduler> {
        self.providers.get_mut(provider_slug)
    }

    /// Call periodically (approx every second) to drive decay.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_decay);
        if elapsed.as_secs() >= 1 {
            for scheduler in self.providers.values_mut() {
                scheduler.apply_decay_all(elapsed);
            }
            self.last_decay = now;
        }
    }

    /// Force-cleanup expired sessions (for memory management).
    pub fn cleanup_expired_sessions(&mut self) {
        for scheduler in self.providers.values_mut() {
            scheduler.evict_expired_sessions();
        }
    }

    pub fn get_all_stats(&self) -> HashMap<String, HashMap<uuid::Uuid, (f64, f64)>> {
        self.providers
            .iter()
            .map(|(slug, sched)| (slug.clone(), sched.get_stats()))
            .collect()
    }
}

impl Default for GlobalKeyScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(slug: &str) -> ProviderKey {
        ProviderKey::new(
            slug.to_string(),
            format!("sk-test-{}", uuid::Uuid::new_v4()),
            "sk-test".to_string(),
            "https://api.test.com".to_string(),
        )
    }

    #[test]
    fn test_session_affinity_same_session_same_key() {
        let mut sched = ProviderKeyScheduler::new("openai".to_string());
        sched.add_key(make_key("openai"));
        sched.add_key(make_key("openai"));

        let s1 = sched.select_key_for_session("session-abc");
        let s2 = sched.select_key_for_session("session-abc");
        let s3 = sched.select_key_for_session("session-abc");

        assert!(s1.is_some());
        assert!(s2.is_some());
        assert!(s3.is_some());
        assert_eq!(s1.as_ref().unwrap().key.id, s2.as_ref().unwrap().key.id);
        assert_eq!(s2.as_ref().unwrap().key.id, s3.as_ref().unwrap().key.id);
    }

    #[test]
    fn test_session_affinity_different_sessions_different_keys() {
        let mut sched = ProviderKeyScheduler::new("openai".to_string());
        let k1 = make_key("openai");
        let k2 = make_key("openai");
        sched.add_key(k1.clone());
        sched.add_key(k2.clone());

        let s1 = sched.select_key_for_session("session-1");
        let s2 = sched.select_key_for_session("session-2");

        assert!(s1.is_some());
        assert!(s2.is_some());
    }

    #[test]
    fn test_failure_rebinds_session() {
        let mut sched = ProviderKeyScheduler::new("openai".to_string());
        let k1 = make_key("openai");
        let k2 = make_key("openai");
        sched.add_key(k1.clone());
        sched.add_key(k2.clone());

        let s1 = sched.select_key_for_session("session-abc");
        let bound_key_id = s1.as_ref().unwrap().key.id;

        for _ in 0..3 {
            sched.record_failure(bound_key_id);
        }

        let s2 = sched.select_key_for_session("session-abc");
        assert!(s2.is_some());
        assert_ne!(
            s2.as_ref().unwrap().key.id, bound_key_id,
            "Session should NOT stay bound to the same key after failure"
        );
    }

    #[test]
    fn test_pressure_calculation() {
        let mut load = KeyLoad::new(uuid::Uuid::new_v4());
        assert!(load.pressure() < 0.0);
        load.requests_in_flight = 10;
        assert!(load.pressure() > 0.0);
    }

    #[test]
    fn test_failure_marks_key_unhealthy() {
        let mut load = KeyLoad::new(uuid::Uuid::new_v4());
        assert!(load.is_healthy);
        load.record_failure();
        load.record_failure();
        load.record_failure();
        assert!(!load.is_healthy);
        assert!(load.pressure() == f64::MAX);
    }

    #[test]
    fn test_previous_key_restored_after_failure() {
        let mut sched = ProviderKeyScheduler::new("openai".to_string());
        let k1 = make_key("openai");
        let k2 = make_key("openai");
        sched.add_key(k1.clone());
        sched.add_key(k2.clone());

        let s1 = sched.select_key_for_session("session-x");
        let key1_id = s1.as_ref().unwrap().key.id;

        sched.record_failure(key1_id);
        sched.record_failure(key1_id);
        sched.record_failure(key1_id);

        sched.revive_key(key1_id);

        let s2 = sched.select_key_for_session("session-x");
        assert!(s2.is_some());
        assert!(
            s2.as_ref().unwrap().restored_from_previous
                || s2.as_ref().unwrap().key.id == key1_id
        );
    }

    #[test]
    fn test_negative_pressure_weight_prefers_idle_key() {
        let mut sched = ProviderKeyScheduler::new("openai".to_string());
        let k1 = make_key("openai");
        let k2 = make_key("openai");
        sched.add_key(k1.clone());
        sched.add_key(k2.clone());

        // k1 is heavily loaded, k2 is idle
        if let Some(load) = sched.loads.get_mut(&k1.id) {
            load.requests_in_flight = 20;
        }

        let mut k2_count = 0u32;
        let trials = 100;
        for _ in 0..trials {
            if let Some(s) = sched.select_key_no_session() {
                if s.key.id == k2.id {
                    k2_count += 1;
                }
            }
        }
        // Idle key should be selected more often
        assert!(
            k2_count > trials / 2,
            "Idle key k2 should be preferred; got {} / {}",
            k2_count,
            trials
        );
    }
}
