use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::DbError;
use models::{User, ApiKey, Provider, ProviderKey, LlmModel, Subscription, Transaction, ApiLog, SubscriptionPlan};
use models::subscription::{TransactionType, TransactionStatus, SubscriptionStatus};
use models::{BrowserAccount, BrowserAccountStatus};

/// PostgreSQL connection pool
pub struct PostgresPool(PgPool);

impl PostgresPool {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        Ok(Self(pool))
    }

    pub fn inner(&self) -> &PgPool {
        &self.0
    }

    // ============ User operations ============

    pub async fn create_user(&self, user: &User) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, phone, password_hash, subscription_plan, 
                              subscription_start, subscription_end, is_admin, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.password_hash)
        .bind(user.subscription_plan.as_str())
        .bind(user.subscription_start)
        .bind(user.subscription_end)
        .bind(user.is_admin)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(self.inner())
            .await?;
        
        Ok(row.map(|r| self.row_to_user(&r)))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(self.inner())
            .await?;

        Ok(row.map(|r| self.row_to_user(&r)))
    }

    pub async fn get_user_by_phone(&self, phone: &str) -> Result<Option<User>, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM users WHERE phone = $1")
            .bind(phone)
            .fetch_optional(self.inner())
            .await?;

        Ok(row.map(|r| self.row_to_user(&r)))
    }

    pub async fn update_user_subscription(
        &self,
        user_id: Uuid,
        plan: SubscriptionPlan,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE users SET subscription_plan = $2, subscription_start = $3, 
                           subscription_end = $4, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(plan.as_str())
        .bind(start)
        .bind(end)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    fn row_to_user(&self, row: &PgRow) -> User {
        User {
            id: row.get("id"),
            email: row.get("email"),
            phone: row.get("phone"),
            password_hash: row.get("password_hash"),
            subscription_plan: SubscriptionPlan::from_str(&row.get::<String, _>("subscription_plan")),
            subscription_start: row.get("subscription_start"),
            subscription_end: row.get("subscription_end"),
            is_admin: row.get("is_admin"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    // ============ API Key operations ============

    pub async fn create_api_key(&self, key: &ApiKey) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, user_id, key_hash, key_prefix, name, is_active, last_used_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(key.id)
        .bind(key.user_id)
        .bind(&key.key_hash)
        .bind(&key.key_prefix)
        .bind(&key.name)
        .bind(key.is_active)
        .bind(key.last_used_at)
        .bind(key.created_at)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true")
            .bind(key_hash)
            .fetch_optional(self.inner())
            .await?;
        
        Ok(row.map(|r| self.row_to_api_key(&r)))
    }

    pub async fn list_api_keys_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>, DbError> {
        let rows = sqlx::query("SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(self.inner())
            .await?;
        
        Ok(rows.iter().map(|r| self.row_to_api_key(r)).collect())
    }

    pub async fn delete_api_key(&self, key_id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(key_id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    pub async fn update_api_key_last_used(&self, key_id: Uuid) -> Result<(), DbError> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    fn row_to_api_key(&self, row: &PgRow) -> ApiKey {
        ApiKey {
            id: row.get("id"),
            user_id: row.get("user_id"),
            key_hash: row.get("key_hash"),
            key_prefix: row.get("key_prefix"),
            name: row.get("name"),
            is_active: row.get("is_active"),
            last_used_at: row.get("last_used_at"),
            created_at: row.get("created_at"),
        }
    }

    // ============ Provider operations ============

    pub async fn create_provider(&self, provider: &Provider) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO providers (id, name, slug, logo_url, api_base_url, is_active, priority, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(provider.id)
        .bind(&provider.name)
        .bind(&provider.slug)
        .bind(&provider.logo_url)
        .bind(&provider.api_base_url)
        .bind(provider.is_active)
        .bind(provider.priority)
        .bind(provider.created_at)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    pub async fn list_providers(&self) -> Result<Vec<Provider>, DbError> {
        let rows = sqlx::query("SELECT * FROM providers WHERE is_active = true ORDER BY priority")
            .fetch_all(self.inner())
            .await?;
        
        Ok(rows.iter().map(|r| self.row_to_provider(r)).collect())
    }

    fn row_to_provider(&self, row: &PgRow) -> Provider {
        Provider {
            id: row.get("id"),
            name: row.get("name"),
            slug: row.get("slug"),
            logo_url: row.get("logo_url"),
            api_base_url: row.get("api_base_url"),
            is_active: row.get("is_active"),
            priority: row.get("priority"),
            created_at: row.get("created_at"),
        }
    }

    // ============ Model operations ============

    pub async fn create_model(&self, model: &LlmModel) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO models (id, provider_id, name, slug, model_id, mode, context_window, 
                              capabilities, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(model.id)
        .bind(&model.provider_id)
        .bind(&model.name)
        .bind(&model.slug)
        .bind(&model.model_id)
        .bind(model.mode.as_str())
        .bind(model.context_window)
        .bind(serde_json::to_value(&model.capabilities)?)
        .bind(model.is_active)
        .bind(model.created_at)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    pub async fn list_models(&self) -> Result<Vec<LlmModel>, DbError> {
        let rows = sqlx::query("SELECT * FROM models WHERE is_active = true ORDER BY name")
            .fetch_all(self.inner())
            .await?;
        
        Ok(rows.iter().map(|r| self.row_to_model(r)).collect())
    }

    pub async fn list_models_by_provider(&self, provider_slug: &str) -> Result<Vec<LlmModel>, DbError> {
        let rows = sqlx::query(
            "SELECT m.* FROM models m JOIN providers p ON m.provider_id = p.slug WHERE p.slug = $1 AND m.is_active = true ORDER BY m.name"
        )
        .bind(provider_slug)
        .fetch_all(self.inner())
        .await?;
        
        Ok(rows.iter().map(|r| self.row_to_model(r)).collect())
    }

    pub async fn get_model_by_slug(&self, slug: &str) -> Result<Option<LlmModel>, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM models WHERE slug = $1 AND is_active = true")
            .bind(slug)
            .fetch_optional(self.inner())
            .await?;
        
        Ok(row.map(|r| self.row_to_model(&r)))
    }

    fn row_to_model(&self, row: &PgRow) -> LlmModel {
        use models::ModelMode;
        
        LlmModel {
            id: row.get("id"),
            provider_id: row.get("provider_id"),
            name: row.get("name"),
            slug: row.get("slug"),
            model_id: row.get("model_id"),
            mode: match row.get::<String, _>("mode").as_str() {
                "completion" => ModelMode::Completion,
                "embedding" => ModelMode::Embedding,
                _ => ModelMode::Chat,
            },
            context_window: row.get("context_window"),
            capabilities: row.get("capabilities"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
        }
    }

    // ============ Subscription operations ============

    pub async fn create_subscription(&self, sub: &Subscription) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO subscriptions (id, user_id, plan, status, start_at, end_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(sub.id)
        .bind(sub.user_id)
        .bind(sub.plan.as_str())
        .bind(format!("{:?}", sub.status).to_lowercase())
        .bind(sub.start_at)
        .bind(sub.end_at)
        .bind(sub.created_at)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    pub async fn get_active_subscription(&self, user_id: Uuid) -> Result<Option<Subscription>, DbError> {
        let row: Option<PgRow> = sqlx::query(
            r#"
            SELECT * FROM subscriptions 
            WHERE user_id = $1 AND status = 'active' AND end_at > NOW()
            ORDER BY created_at DESC LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.inner())
        .await?;
        
        Ok(row.map(|r| self.row_to_subscription(&r)))
    }

    fn row_to_subscription(&self, row: &PgRow) -> Subscription {
        use models::subscription::SubscriptionStatus;
        use models::SubscriptionPlan;
        
        Subscription {
            id: row.get("id"),
            user_id: row.get("user_id"),
            plan: SubscriptionPlan::from_str(&row.get::<String, _>("plan")),
            status: match row.get::<String, _>("status").as_str() {
                "expired" => SubscriptionStatus::Expired,
                "cancelled" => SubscriptionStatus::Cancelled,
                _ => SubscriptionStatus::Active,
            },
            start_at: row.get("start_at"),
            end_at: row.get("end_at"),
            created_at: row.get("created_at"),
        }
    }

    // ============ Transaction operations ============

    pub async fn create_transaction(&self, tx: &Transaction) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO transactions (id, user_id, transaction_type, amount, plan, status, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(tx.id)
        .bind(tx.user_id)
        .bind(format!("{:?}", tx.transaction_type).to_lowercase())
        .bind(Decimal::from_f64_retain(tx.amount).unwrap_or_default())
        .bind(tx.plan.as_ref().map(|p: &SubscriptionPlan| p.as_str()))
        .bind(format!("{:?}", tx.status).to_lowercase())
        .bind(&tx.description)
        .bind(tx.created_at)
        .execute(self.inner())
        .await?;
        
        Ok(())
    }

    pub async fn list_transactions(&self, user_id: Uuid, limit: i32) -> Result<Vec<Transaction>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.inner())
        .await?;
        
        Ok(rows.iter().map(|r| self.row_to_transaction(r)).collect())
    }

    fn row_to_transaction(&self, row: &PgRow) -> Transaction {
        use models::subscription::{TransactionStatus, TransactionType};
        use models::SubscriptionPlan;
        
        Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            transaction_type: match row.get::<String, _>("transaction_type").as_str() {
                "subscription_purchase" => TransactionType::SubscriptionPurchase,
                "subscription_renewal" => TransactionType::SubscriptionRenewal,
                "subscription_cancellation" => TransactionType::SubscriptionCancellation,
                _ => TransactionType::Refund,
            },
            amount: rust_decimal::prelude::ToPrimitive::to_f64(&row.get::<Decimal, _>("amount")).unwrap_or(0.0),
            plan: row.get::<Option<String>, _>("plan").map(|p| SubscriptionPlan::from_str(&p)),
            status: match row.get::<String, _>("status").as_str() {
                "pending" => TransactionStatus::Pending,
                "failed" => TransactionStatus::Failed,
                "refunded" => TransactionStatus::Refunded,
                _ => TransactionStatus::Completed,
            },
            description: row.get("description"),
            created_at: row.get("created_at"),
        }
    }

    // ============ API Log operations ============

    pub async fn create_api_log(&self, log: &ApiLog) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO api_logs (id, user_id, api_key_id, provider_id, model_id, mode,
                                 input_tokens, output_tokens, latency_ms, status, error_message, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(log.id)
        .bind(log.user_id)
        .bind(log.api_key_id)
        .bind(&log.provider_id)
        .bind(&log.model_id)
        .bind(&log.mode)
        .bind(log.input_tokens)
        .bind(log.output_tokens)
        .bind(log.latency_ms)
        .bind(format!("{:?}", log.status).to_lowercase())
        .bind(&log.error_message)
        .bind(log.created_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    /// Get total tokens used by a user in the current billing cycle
    /// Billing cycle starts at subscription_start
    pub async fn get_user_token_usage_in_period(
        &self,
        user_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(input_tokens + output_tokens), 0) as total_tokens
            FROM api_logs
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at < $3
              AND status = 'success'
            "#,
        )
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(self.inner())
        .await?;

        let total: i64 = row.get("total_tokens");
        Ok(total)
    }

    /// Get usage statistics for a user in a period
    pub async fn get_user_usage_stats(
        &self,
        user_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<models::UsageStats, DbError> {
        // Total stats
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_requests,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COALESCE(SUM(latency_ms), 0) as total_latency_ms
            FROM api_logs
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at < $3
              AND status = 'success'
            "#,
        )
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(self.inner())
        .await?;

        // Usage by provider
        let provider_rows = sqlx::query(
            r#"
            SELECT provider_id,
                   COUNT(*) as requests,
                   COALESCE(SUM(input_tokens), 0) as input_tokens,
                   COALESCE(SUM(output_tokens), 0) as output_tokens
            FROM api_logs
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at < $3
              AND status = 'success'
            GROUP BY provider_id
            ORDER BY requests DESC
            "#,
        )
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(self.inner())
        .await?;

        // Usage by model
        let model_rows = sqlx::query(
            r#"
            SELECT model_id, provider_id,
                   COUNT(*) as requests,
                   COALESCE(SUM(input_tokens), 0) as input_tokens,
                   COALESCE(SUM(output_tokens), 0) as output_tokens
            FROM api_logs
            WHERE user_id = $1
              AND created_at >= $2
              AND created_at < $3
              AND status = 'success'
            GROUP BY model_id, provider_id
            ORDER BY requests DESC
            "#,
        )
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(self.inner())
        .await?;

        Ok(models::UsageStats {
            user_id,
            period_start,
            period_end,
            total_requests: row.get("total_requests"),
            total_input_tokens: row.get("total_input_tokens"),
            total_output_tokens: row.get("total_output_tokens"),
            total_latency_ms: row.get("total_latency_ms"),
            usage_by_provider: provider_rows.iter().map(|r| models::ProviderUsage {
                provider: r.get("provider_id"),
                requests: r.get("requests"),
                input_tokens: r.get("input_tokens"),
                output_tokens: r.get("output_tokens"),
            }).collect(),
            usage_by_model: model_rows.iter().map(|r| models::ModelUsage {
                model: r.get("model_id"),
                provider: r.get("provider_id"),
                requests: r.get("requests"),
                input_tokens: r.get("input_tokens"),
                output_tokens: r.get("output_tokens"),
            }).collect(),
        })
    }

    // ============ Admin operations ============

    pub async fn list_users(&self, offset: i64, limit: i64, search: &str) -> Result<Vec<User>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM users
            WHERE ($1 = '' OR email ILIKE '%' || $1 || '%')
            ORDER BY created_at DESC
            OFFSET $2 LIMIT $3
            "#,
        )
        .bind(search)
        .bind(offset)
        .bind(limit)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_user(r)).collect())
    }

    pub async fn count_users(&self, search: &str) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM users
            WHERE ($1 = '' OR email ILIKE '%' || $1 || '%')
            "#,
        )
        .bind(search)
        .fetch_one(self.inner())
        .await?;

        Ok(row.get("count"))
    }

    pub async fn update_user_admin(
        &self,
        user_id: Uuid,
        phone: Option<&str>,
        subscription_plan: Option<&str>,
    ) -> Result<(), DbError> {
        if let Some(phone) = phone {
            sqlx::query("UPDATE users SET phone = $2, updated_at = NOW() WHERE id = $1")
                .bind(user_id)
                .bind(phone)
                .execute(self.inner())
                .await?;
        }
        if let Some(plan) = subscription_plan {
            sqlx::query("UPDATE users SET subscription_plan = $2, updated_at = NOW() WHERE id = $1")
                .bind(user_id)
                .bind(plan)
                .execute(self.inner())
                .await?;
        }
        Ok(())
    }

    pub async fn list_all_providers(&self) -> Result<Vec<Provider>, DbError> {
        let rows = sqlx::query("SELECT * FROM providers ORDER BY priority")
            .fetch_all(self.inner())
            .await?;

        Ok(rows.iter().map(|r| self.row_to_provider(r)).collect())
    }

    pub async fn update_provider(
        &self,
        id: Uuid,
        name: Option<&str>,
        slug: Option<&str>,
        api_base_url: Option<&str>,
        is_active: Option<bool>,
        priority: Option<i32>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE providers SET
                name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                api_base_url = COALESCE($4, api_base_url),
                is_active = COALESCE($5, is_active),
                priority = COALESCE($6, priority)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(api_base_url)
        .bind(is_active)
        .bind(priority)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_provider_soft(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("UPDATE providers SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    // ============ Provider Key operations ============

    pub async fn create_provider_key(&self, key: &ProviderKey) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO provider_keys (id, provider_slug, api_key_encrypted, api_key_prefix,
                                       base_url, is_active, priority, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(key.id)
        .bind(&key.provider_slug)
        .bind(&key.api_key_encrypted)
        .bind(&key.api_key_prefix)
        .bind(&key.base_url)
        .bind(key.is_active)
        .bind(key.priority)
        .bind(key.created_at)
        .bind(key.updated_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn list_provider_keys(&self) -> Result<Vec<ProviderKey>, DbError> {
        let rows = sqlx::query("SELECT * FROM provider_keys ORDER BY priority")
            .fetch_all(self.inner())
            .await?;

        Ok(rows.iter().map(|r| self.row_to_provider_key(r)).collect())
    }

    pub async fn get_provider_key_by_id(&self, id: Uuid) -> Result<Option<ProviderKey>, DbError> {
        let row = sqlx::query("SELECT * FROM provider_keys WHERE id = $1")
            .bind(id)
            .fetch_optional(self.inner())
            .await?;

        Ok(row.map(|r| self.row_to_provider_key(&r)))
    }

    pub async fn get_provider_key_by_slug(&self, slug: &str) -> Result<Option<ProviderKey>, DbError> {
        let row = sqlx::query("SELECT * FROM provider_keys WHERE provider_slug = $1 AND is_active = true")
            .bind(slug)
            .fetch_optional(self.inner())
            .await?;

        Ok(row.map(|r| self.row_to_provider_key(&r)))
    }

    pub async fn update_provider_key(
        &self,
        id: Uuid,
        api_key_encrypted: Option<&str>,
        api_key_prefix: Option<&str>,
        base_url: Option<&str>,
        is_active: Option<bool>,
        priority: Option<i32>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE provider_keys SET
                api_key_encrypted = COALESCE($2, api_key_encrypted),
                api_key_prefix = COALESCE($3, api_key_prefix),
                base_url = COALESCE($4, base_url),
                is_active = COALESCE($5, is_active),
                priority = COALESCE($6, priority),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(api_key_encrypted)
        .bind(api_key_prefix)
        .bind(base_url)
        .bind(is_active)
        .bind(priority)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_provider_key(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM provider_keys WHERE id = $1")
            .bind(id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    fn row_to_provider_key(&self, row: &PgRow) -> ProviderKey {
        ProviderKey {
            id: row.get("id"),
            provider_slug: row.get("provider_slug"),
            api_key_encrypted: row.get("api_key_encrypted"),
            api_key_prefix: row.get("api_key_prefix"),
            base_url: row.get("base_url"),
            is_active: row.get("is_active"),
            priority: row.get("priority"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub async fn list_all_models(&self) -> Result<Vec<LlmModel>, DbError> {
        let rows = sqlx::query("SELECT * FROM models ORDER BY name")
            .fetch_all(self.inner())
            .await?;

        Ok(rows.iter().map(|r| self.row_to_model(r)).collect())
    }

    pub async fn update_model(
        &self,
        id: Uuid,
        name: Option<&str>,
        slug: Option<&str>,
        model_id: Option<&str>,
        context_window: Option<i32>,
        capabilities: Option<serde_json::Value>,
        is_active: Option<bool>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE models SET
                name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                model_id = COALESCE($4, model_id),
                context_window = COALESCE($5, context_window),
                capabilities = COALESCE($6, capabilities),
                is_active = COALESCE($7, is_active)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(model_id)
        .bind(context_window)
        .bind(capabilities)
        .bind(is_active)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_model_soft(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("UPDATE models SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    pub async fn list_all_transactions(
        &self,
        offset: i64,
        limit: i64,
        tx_type: &str,
        status: &str,
    ) -> Result<Vec<(Transaction, String)>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT t.*, u.email as user_email
            FROM transactions t
            JOIN users u ON t.user_id = u.id
            WHERE ($3 = '' OR t.transaction_type = $3)
              AND ($4 = '' OR t.status = $4)
            ORDER BY t.created_at DESC
            OFFSET $1 LIMIT $2
            "#,
        )
        .bind(offset)
        .bind(limit)
        .bind(tx_type)
        .bind(status)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| {
            let tx = self.row_to_transaction(r);
            let email: String = r.get("user_email");
            (tx, email)
        }).collect())
    }

    pub async fn count_all_transactions(&self, tx_type: &str, status: &str) -> Result<i64, DbError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM transactions
            WHERE ($1 = '' OR transaction_type = $1)
              AND ($2 = '' OR status = $2)
            "#,
        )
        .bind(tx_type)
        .bind(status)
        .fetch_one(self.inner())
        .await?;

        Ok(row.get("count"))
    }

    pub async fn get_dashboard_stats(&self) -> Result<DashboardStats, DbError> {
        let user_count: i64 = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(self.inner())
            .await?
            .get("count");

        let active_subs: i64 = sqlx::query(
            "SELECT COUNT(*) as count FROM users WHERE subscription_plan != 'none' AND subscription_end > NOW()"
        )
        .fetch_one(self.inner())
        .await?
        .get("count");

        let total_revenue: f64 = sqlx::query(
            r#"SELECT COALESCE(SUM(amount), 0) as total FROM transactions WHERE status = 'completed' AND amount > 0"#
        )
        .fetch_one(self.inner())
        .await?
        .get::<rust_decimal::Decimal, _>("total")
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);

        let api_calls_today: i64 = sqlx::query(
            "SELECT COUNT(*) as count FROM api_logs WHERE created_at >= CURRENT_DATE"
        )
        .fetch_one(self.inner())
        .await?
        .get("count");

        Ok(DashboardStats {
            total_users: user_count,
            active_subscriptions: active_subs,
            total_revenue,
            api_calls_today,
        })
    }

    // ============ Browser Account operations (ZeroToken) ============

    pub async fn create_browser_account(&self, account: &BrowserAccount) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO browser_accounts (id, provider, email, session_data_encrypted, status,
                                          request_count, last_used_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(account.id)
        .bind(&account.provider)
        .bind(&account.email)
        .bind(&account.session_data_encrypted)
        .bind(account.status.as_str())
        .bind(account.request_count)
        .bind(account.last_used_at)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn list_browser_accounts(&self) -> Result<Vec<BrowserAccount>, DbError> {
        let rows = sqlx::query("SELECT * FROM browser_accounts ORDER BY created_at DESC")
            .fetch_all(self.inner())
            .await?;

        Ok(rows.iter().map(|r| self.row_to_browser_account(r)).collect())
    }

    pub async fn get_browser_account(&self, id: Uuid) -> Result<Option<BrowserAccount>, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM browser_accounts WHERE id = $1")
            .bind(id)
            .fetch_optional(self.inner())
            .await?;

        Ok(row.map(|r| self.row_to_browser_account(&r)))
    }

    pub async fn update_browser_account_session(
        &self,
        id: Uuid,
        session_data: &str,
        email: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE browser_accounts
            SET session_data_encrypted = $2,
                email = COALESCE($3, email),
                status = 'active',
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(session_data)
        .bind(email)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn increment_browser_account_request_count(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE browser_accounts
            SET request_count = request_count + 1,
                last_used_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn update_browser_account_status(&self, id: Uuid, status: BrowserAccountStatus) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE browser_accounts SET status = $2, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .bind(status.as_str())
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_browser_account(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM browser_accounts WHERE id = $1")
            .bind(id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    fn row_to_browser_account(&self, row: &PgRow) -> BrowserAccount {
        BrowserAccount {
            id: row.get("id"),
            provider: row.get("provider"),
            email: row.get("email"),
            session_data_encrypted: row.get("session_data_encrypted"),
            status: BrowserAccountStatus::from_str(&row.get::<String, _>("status")),
            request_count: row.get("request_count"),
            last_used_at: row.get("last_used_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub active_subscriptions: i64,
    pub total_revenue: f64,
    pub api_calls_today: i64,
}
