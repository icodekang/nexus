/**
 * @file KeysPage — Credentials Vault
 * Refined minimal key management. Teal accent. Silky transitions.
 */
import { useState, useEffect, useCallback } from 'react';
import {
  Key, Plus, Trash2, Copy, Check, AlertCircle,
  Settings2, ArrowUp, ArrowDown, Zap, Globe, Server
} from 'lucide-react';
import {
  fetchApiKeys, createApiKey, deleteApiKey, type ApiKey,
  fetchProviderKeys, createProviderKey, updateProviderKey, deleteProviderKey, type ProviderKeyItem,
} from '../api/client';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import { getErrorMessage } from '../utils/errors';
import openaiIcon from '../assets/icons/OpenAI.svg';
import anthropicIcon from '../assets/icons/Anthropic.svg';
import geminiIcon from '../assets/icons/GoogleGemini.svg';
import deepseekIcon from '../assets/icons/DeepSeek.png';
import './KeysPage.css';

const PROVIDERS = [
  { slug: 'openai', name: 'OpenAI', color: '#10A37F' },
  { slug: 'anthropic', name: 'Anthropic', color: '#E07B56' },
  { slug: 'google', name: 'Google', color: '#4285F4' },
  { slug: 'deepseek', name: 'DeepSeek', color: '#536DFE' },
];

const PROVIDER_LOGOS: Record<string, string> = {
  openai: openaiIcon,
  anthropic: anthropicIcon,
  google: geminiIcon,
  deepseek: deepseekIcon,
};

type Tab = 'nexus' | 'byok';

export default function KeysPage() {
  const { t, locale } = useI18n();
  const { isAuthenticated, requireAuth } = useAuthStore();

  const [tab, setTab] = useState<Tab>('nexus');

  const [nexKeys, setNexKeys] = useState<ApiKey[]>([]);
  const [nexLoading, setNexLoading] = useState(true);
  const [nexCreating, setNexCreating] = useState(false);
  const [nexName, setNexName] = useState('');
  const [nexShowForm, setNexShowForm] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [nexToDelete, setNexToDelete] = useState<ApiKey | null>(null);

  const [provKeys, setProvKeys] = useState<ProviderKeyItem[]>([]);
  const [provLoading, setProvLoading] = useState(true);
  const [provShowForm, setProvShowForm] = useState(false);
  const [provProvider, setProvProvider] = useState('openai');
  const [provKey, setProvKey] = useState('');
  const [provName, setProvName] = useState('');
  const [provPriority, setProvPriority] = useState('prioritized');
  const [provAlwaysUse, setProvAlwaysUse] = useState(false);
  const [provSubmitting, setProvSubmitting] = useState(false);
  const [provDeleting, setProvDeleting] = useState<string | null>(null);

  const [error, setError] = useState('');

  const loadNexKeys = useCallback(async () => {
    if (!isAuthenticated) { setNexLoading(false); return; }
    try {
      const res = await fetchApiKeys();
      setNexKeys(res.data);
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally { setNexLoading(false); }
  }, [isAuthenticated]);

  useEffect(() => { loadNexKeys(); }, [loadNexKeys]);

  const loadProvKeys = useCallback(async () => {
    if (!isAuthenticated) { setProvLoading(false); return; }
    try {
      const res = await fetchProviderKeys();
      setProvKeys(res.data);
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally { setProvLoading(false); }
  }, [isAuthenticated]);

  useEffect(() => { loadProvKeys(); }, [loadProvKeys]);

  const handleNexCreate = async () => {
    if (!nexName.trim()) return;
    if (!requireAuth()) return;
    setNexCreating(true); setError('');
    try {
      const res = await createApiKey(nexName.trim());
      setNewKey(res.key);
      setNexName(''); setNexShowForm(false);
    } catch (err: unknown) { setError(getErrorMessage(err, t)); }
    finally { setNexCreating(false); }
  };

  const handleNexDelete = async () => {
    if (!nexToDelete) return;
    try {
      await deleteApiKey(nexToDelete.id);
      setNexKeys(p => p.filter(k => k.id !== nexToDelete.id));
    } catch (err: unknown) { setError(getErrorMessage(err, t)); }
    finally { setNexToDelete(null); }
  };

  const handleProvCreate = async () => {
    if (!provKey.trim()) return;
    if (!requireAuth()) return;
    setProvSubmitting(true); setError('');
    try {
      await createProviderKey({
        provider_slug: provProvider,
        api_key: provKey.trim(),
        name: provName.trim() || undefined,
        priority_level: provPriority,
        always_use: provAlwaysUse,
      });
      setProvKey(''); setProvName(''); setProvShowForm(false);
      await loadProvKeys();
    } catch (err: unknown) { setError(getErrorMessage(err, t)); }
    finally { setProvSubmitting(false); }
  };

  const handleProvTogglePriority = async (k: ProviderKeyItem) => {
    try {
      await updateProviderKey(k.id, {
        priority_level: k.priority_level === 'prioritized' ? 'fallback' : 'prioritized',
      });
      await loadProvKeys();
    } catch (err: unknown) { setError(getErrorMessage(err, t)); }
  };

  const handleProvToggleAlways = async (k: ProviderKeyItem) => {
    try {
      await updateProviderKey(k.id, { always_use: !k.always_use });
      await loadProvKeys();
    } catch (err: unknown) { setError(getErrorMessage(err, t)); }
  };

  const handleProvDelete = async (id: string) => {
    setProvDeleting(id);
    try { await deleteProviderKey(id); await loadProvKeys(); }
    catch (err: unknown) { setError(getErrorMessage(err, t)); }
    finally { setProvDeleting(null); }
  };

  const handleCopy = async (text: string) => {
    try { await navigator.clipboard.writeText(text); } catch {
      const ta = document.createElement('textarea');
      ta.value = text; ta.style.position = 'fixed'; ta.style.opacity = '0';
      document.body.appendChild(ta); ta.select();
      document.execCommand('copy'); document.body.removeChild(ta);
    }
    setCopied(true); setTimeout(() => setCopied(false), 2000);
  };

  const fmt = (d: string) =>
    new Date(d).toLocaleDateString(locale === 'zh' ? 'zh-CN' : 'en-US', {
      month: 'short', day: 'numeric', year: 'numeric',
    });

  const switchTab = (t: Tab) => { setTab(t); setError(''); };

  return (
    <div className="keys-page">
      {/* ── Header ── */}
      <header className="keys-header">
        <div>
          <h1 className="keys-title">{t('keys.title')}</h1>
          <p className="keys-subtitle">{t('keys.subtitle')}</p>
        </div>
      </header>

      <div className="keys-banner">
        <Zap size={13} strokeWidth={2} />
        <span>{t('providerKeys.infoBanner')}</span>
      </div>

      {error && (
        <div className="keys-error"><AlertCircle size={14} /><span>{error}</span></div>
      )}

      {/* ── Segment control ── */}
      <div className={`keys-segment ${tab === 'byok' ? 'provider' : ''}`}>
        <button className={`keys-segment-btn ${tab === 'nexus' ? 'active' : ''}`} onClick={() => switchTab('nexus')}>
          <Server size={14} strokeWidth={1.75} />
          <span>{t('keys.tabNexusKeys')}</span>
        </button>
        <button className={`keys-segment-btn ${tab === 'byok' ? 'active' : ''}`} onClick={() => switchTab('byok')}>
          <Globe size={14} strokeWidth={1.75} />
          <span>{t('keys.tabProviderKeys')}</span>
        </button>
      </div>

      {/* ═════════════════════ TAB PANELS ═════════════════════ */}

      {/* NEXUS API KEYS */}
      {tab === 'nexus' && (
        <div className="keys-panel" key="nexus">
          {newKey && (
            <div className="keys-reveal">
              <div className="keys-reveal-top">
                <span className="keys-reveal-label">{t('keys.newKeyReveal')}</span>
                <span className="keys-reveal-warn">{t('keys.copyWarning')}</span>
              </div>
              <div className="keys-reveal-row">
                <code>{newKey}</code>
                <button className={`keys-copy-btn ${copied ? 'done' : ''}`} onClick={() => handleCopy(newKey)}>
                  {copied ? <Check size={14} /> : <Copy size={14} />}
                </button>
                <span className={`keys-copy-feedback ${copied ? 'on' : ''}`}>{t('keys.copiedSuccess')}</span>
              </div>
              <button className="keys-reveal-dismiss" onClick={() => { setNewKey(null); loadNexKeys(); }}>
                {t('common.dismiss')}
              </button>
            </div>
          )}

          <div className="keys-bar">
            <button className="keys-btn keys-btn-primary" onClick={() => { if (requireAuth()) setNexShowForm(!nexShowForm); }}>
              <Plus size={14} strokeWidth={2} />
              {t('keys.newKey')}
            </button>
          </div>

          <div className={`keys-form-panel ${nexShowForm ? 'open' : ''}`}>
            <div className="keys-form-panel-inner">
              <div className="keys-form-wrap">
                <div className="keys-form-row">
                  <input
                    className="keys-input"
                    placeholder={t('keys.keyNamePlaceholder')}
                    value={nexName}
                    onChange={e => setNexName(e.target.value)}
                    onKeyDown={e => e.key === 'Enter' && handleNexCreate()}
                    autoFocus
                  />
                  <button className="keys-btn keys-btn-primary" onClick={handleNexCreate} disabled={nexCreating || !nexName.trim()}>
                    {nexCreating ? t('common.creating') : t('common.create')}
                  </button>
                  <button className="keys-btn keys-btn-outline" onClick={() => setNexShowForm(false)}>{t('common.cancel')}</button>
                </div>
              </div>
            </div>
          </div>

          {nexLoading ? (
            <div className="keys-skeleton">
              <div className="keys-skeleton-line" />
              <div className="keys-skeleton-line" />
              <div className="keys-skeleton-line" />
            </div>
          ) : nexKeys.length === 0 ? (
            <div className="keys-empty">
              <div className="keys-empty-icon"><Key size={22} strokeWidth={1.5} /></div>
              <h3>{t('keys.noKeys')}</h3>
              <p>{t('keys.noKeysDesc')}</p>
            </div>
          ) : (
            <div className="keys-list">
              {nexKeys.map((key, i) => (
                <div key={key.id} className="keys-card" style={{ animationDelay: `${i * 0.04}s` }}>
                  <div className="keys-card-icon"><Key size={14} strokeWidth={1.75} /></div>
                  <div className="keys-card-body">
                    <div className="keys-card-name">{key.name || t('keys.unnamedKey')}</div>
                    <div className="keys-card-meta">
                      <span className="keys-card-prefix">{maskNexKey(key.key_prefix)}</span>
                      <span className="keys-card-date">{t('keys.createdDate', { date: fmt(key.created_at) })}</span>
                      <span className={`keys-status ${key.last_used_at ? 'used' : ''}`} title={key.last_used_at ? t('keys.used') : t('keys.neverUsed')}>
                        <span className="keys-status-dot" />
                      </span>
                    </div>
                  </div>
                  <div className="keys-card-actions">
                    <button className="keys-icon-btn danger" onClick={() => setNexToDelete(key)} title={t('keys.deleteKey')}>
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* PROVIDER KEYS */}
      {tab === 'byok' && (
        <div className="keys-panel" key="byok">
          <div className="keys-bar">
            <button className="keys-btn keys-btn-primary" onClick={() => { if (requireAuth()) { setProvShowForm(!provShowForm); setError(''); } }}>
              <Plus size={14} strokeWidth={2} />
              {t('providerKeys.addKey')}
            </button>
          </div>

          <div className={`keys-form-panel ${provShowForm ? 'open' : ''}`}>
            <div className="keys-form-panel-inner">
              <div className="keys-form-wrap">
                <div className="keys-prov-form-grid">
                  <div className="keys-field">
                    <label>{t('providerKeys.provider')}</label>
                    <select value={provProvider} onChange={e => setProvProvider(e.target.value)}>
                      {PROVIDERS.map(p => <option key={p.slug} value={p.slug}>{p.name}</option>)}
                    </select>
                  </div>
                  <div className="keys-field">
                    <label>{t('providerKeys.apiKey')}</label>
                    <input type="password" value={provKey} onChange={e => setProvKey(e.target.value)}
                      placeholder={t('keys.providerKeyPlaceholder')} autoComplete="off" />
                  </div>
                  <div className="keys-field">
                    <label>{t('providerKeys.name')} <span className="keys-field-opt">({t('common.optional')})</span></label>
                    <input value={provName} onChange={e => setProvName(e.target.value)} placeholder={t('providerKeys.namePlaceholder')} />
                  </div>
                  <div className="keys-field">
                    <label>{t('providerKeys.priority')}</label>
                    <select value={provPriority} onChange={e => setProvPriority(e.target.value)}>
                      <option value="prioritized">{t('providerKeys.prioritized')}</option>
                      <option value="fallback">{t('providerKeys.fallback')}</option>
                    </select>
                  </div>
                </div>
                <label className="keys-prov-checkbox">
                  <input type="checkbox" checked={provAlwaysUse} onChange={e => setProvAlwaysUse(e.target.checked)} />
                  <span>{t('providerKeys.alwaysUse')}</span>
                </label>
                <div className="keys-form-actions">
                  <button className="keys-btn keys-btn-outline" onClick={() => setProvShowForm(false)}>{t('common.cancel')}</button>
                  <button className="keys-btn keys-btn-primary" onClick={handleProvCreate} disabled={provSubmitting}>
                    {provSubmitting ? <span className="keys-spinner" /> : t('providerKeys.save')}
                  </button>
                </div>
              </div>
            </div>
          </div>

          {provLoading ? (
            <div className="keys-skeleton">
              <div className="keys-skeleton-line" />
              <div className="keys-skeleton-line" />
              <div className="keys-skeleton-line" />
            </div>
          ) : provKeys.length === 0 ? (
            <div className="keys-empty">
              <div className="keys-empty-icon"><Globe size={22} strokeWidth={1.5} /></div>
              <h3>{t('providerKeys.noKeys')}</h3>
              <p>{t('providerKeys.noKeysDesc')}</p>
            </div>
          ) : (
            PROVIDERS.map(prov => {
              const group = provKeys.filter(k => k.provider_slug === prov.slug);
              if (group.length === 0) return null;
              return (
                <div key={prov.slug} className="keys-group">
                  <div className="keys-group-head">
                    <span className="keys-group-dot" style={{ background: prov.color }} />
                    <span className="keys-group-name">{prov.name}</span>
                    <span className="keys-group-count">{group.length}</span>
                  </div>
                  <div className="keys-list">
                    {group.sort((a, b) => a.sort_order - b.sort_order).map((k, i) => (
                      <div key={k.id} className="keys-card" style={{ animationDelay: `${i * 0.04}s` }}>
                        <div className="keys-card-icon keys-card-icon-logo">
                          <img src={PROVIDER_LOGOS[prov.slug]} alt={prov.name} />
                        </div>
                        <div className="keys-card-body">
                          <div className="keys-card-name">
                            {k.name || t('providerKeys.unnamed')}
                            <span className={`keys-tag ${k.priority_level}`}>
                              {k.priority_level === 'prioritized' ? t('providerKeys.prioritized') : t('providerKeys.fallback')}
                            </span>
                            {k.always_use && <span className="keys-tag lock">{t('providerKeys.alwaysUse')}</span>}
                          </div>
                          <div className="keys-card-meta">
                            <span className="keys-card-prefix">{maskProvKey(k.api_key_prefix)}</span>
                            <span className="keys-card-date">{fmt(k.created_at)}</span>
                          </div>
                        </div>
                        <div className="keys-card-actions">
                          <button className={`keys-icon-btn ${k.priority_level === 'prioritized' ? 'on' : ''}`}
                            onClick={() => handleProvTogglePriority(k)}
                            title={k.priority_level === 'prioritized' ? t('providerKeys.setFallback') : t('providerKeys.setPrioritized')}>
                            {k.priority_level === 'prioritized' ? <ArrowUp size={14} /> : <ArrowDown size={14} />}
                          </button>
                          <button className={`keys-icon-btn ${k.always_use ? 'on' : ''}`}
                            onClick={() => handleProvToggleAlways(k)} title={t('providerKeys.alwaysUse')}>
                            <Settings2 size={14} />
                          </button>
                          <button className="keys-icon-btn danger"
                            onClick={() => handleProvDelete(k.id)} disabled={provDeleting === k.id}>
                            <Trash2 size={14} />
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      {/* ── Delete confirm dialog ── */}
      {nexToDelete && (
        <div className="keys-overlay" onClick={() => setNexToDelete(null)}>
          <div className="keys-dialog" onClick={e => e.stopPropagation()}>
            <h3>{t('keys.deleteConfirmTitle')}</h3>
            <p>{t('keys.deleteConfirmDesc')}</p>
            <div className="keys-dialog-name">{nexToDelete.name || t('keys.unnamedKey')}</div>
            <div className="keys-dialog-actions">
              <button className="keys-btn keys-btn-outline" onClick={() => setNexToDelete(null)}>{t('common.cancel')}</button>
              <button className="keys-btn keys-btn-danger" onClick={handleNexDelete}>{t('keys.confirmDelete')}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function maskNexKey(prefix: string): string {
  return prefix.replace(/^(sk-nexus-)(.+)$/, (_, p, rest) => {
    if (rest.length <= 5) return p + rest[0] + '*'.repeat(rest.length - 1);
    return p + rest[0] + '*'.repeat(rest.length - 5) + rest.slice(-4);
  });
}

function maskProvKey(prefix: string): string {
  if (prefix.length <= 8) return prefix.slice(0, 4) + '*'.repeat(9);
  return prefix.slice(0, 4) + '*'.repeat(9) + prefix.slice(-4);
}
