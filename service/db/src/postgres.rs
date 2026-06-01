//! PostgreSQL 数据库模块

use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use uuid::Uuid;

use crate::DbError;
use models::{
    ApiKey, ApiLog, LlmModel, ModelPricing, PriorityLevel, PricingMode, Provider,
    ProviderKey, TokenCharge, TokenPackage, Transaction, User, UserBalance, UserProviderKey,
};

/// PostgreSQL 连接池
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

    pub fn pool(&self) -> &PgPool {
        &self.0
    }

    // ============ User operations ============

    pub async fn create_user(&self, user: &User) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, phone, password_hash, is_admin, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.password_hash)
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

    fn row_to_user(&self, row: &PgRow) -> User {
        User {
            id: row.get("id"),
            email: row.get("email"),
            phone: row.get("phone"),
            password_hash: row.get("password_hash"),
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
        .await
        .map_err(|e| {
            tracing::error!("create_api_key failed: id={}, user_id={}, key_prefix={}, error={:?}", key.id, key.user_id, key.key_prefix, e);
            e
        })?;

        Ok(())
    }

    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DbError> {
        let row: Option<PgRow> =
            sqlx::query("SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true")
                .bind(key_hash)
                .fetch_optional(self.inner())
                .await?;

        Ok(row.map(|r| self.row_to_api_key(&r)))
    }

    pub async fn list_api_keys_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>, DbError> {
        let rows =
            sqlx::query("SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC")
                .bind(user_id)
                .fetch_all(self.inner())
                .await?;

        Ok(rows.iter().map(|r| self.row_to_api_key(r)).collect())
    }

    pub async fn list_all_api_keys_with_users(&self) -> Result<Vec<(ApiKey, String)>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT ak.*, u.email as user_email
            FROM api_keys ak
            JOIN users u ON ak.user_id = u.id
            ORDER BY ak.created_at DESC
            "#,
        )
        .fetch_all(self.inner())
        .await?;

        Ok(rows
            .iter()
            .map(|r| {
                let api_key = ApiKey {
                    id: r.get("id"),
                    user_id: r.get("user_id"),
                    key_hash: r.get("key_hash"),
                    key_prefix: r.get("key_prefix"),
                    name: r.get("name"),
                    is_active: r.get("is_active"),
                    last_used_at: r.get("last_used_at"),
                    created_at: r.get("created_at"),
                };
                let user_email: String = r.get("user_email");
                (api_key, user_email)
            })
            .collect())
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
            INSERT INTO providers (id, name, slug, logo_url, api_base_url, api_type, openai_api_url, anthropic_api_url, is_active, priority, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(provider.id)
        .bind(&provider.name)
        .bind(&provider.slug)
        .bind(&provider.logo_url)
        .bind(&provider.api_base_url)
        .bind(&provider.api_type)
        .bind(&provider.openai_api_url)
        .bind(&provider.anthropic_api_url)
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
            api_type: row.get("api_type"),
            openai_api_url: row.get("openai_api_url"),
            anthropic_api_url: row.get("anthropic_api_url"),
            is_active: row.get("is_active"),
            priority: row.get("priority"),
            created_at: row.get("created_at"),
        }
    }

    // ============ Model operations ============

    pub async fn create_model(&self, model: &LlmModel) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO models (id, provider_id, name, model_id, mode, context_window, 
                              capabilities, description, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(model.id)
        .bind(&model.provider_id)
        .bind(&model.name)
        .bind(&model.model_id)
        .bind(model.mode.as_str())
        .bind(model.context_window)
        .bind(serde_json::to_value(&model.capabilities)?)
        .bind(&model.description)
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

    pub async fn list_models_by_provider(
        &self,
        provider_slug: &str,
    ) -> Result<Vec<LlmModel>, DbError> {
        let rows = sqlx::query(
            "SELECT m.* FROM models m JOIN providers p ON m.provider_id = p.slug WHERE p.slug = $1 AND m.is_active = true ORDER BY m.name"
        )
        .bind(provider_slug)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_model(r)).collect())
    }

    pub async fn get_model_by_id(&self, model_id: &str) -> Result<Option<LlmModel>, DbError> {
        let row: Option<PgRow> =
            sqlx::query("SELECT * FROM models WHERE model_id = $1 AND is_active = true")
                .bind(model_id)
                .fetch_optional(self.inner())
                .await?;

        Ok(row.map(|r| self.row_to_model(&r)))
    }

    fn row_to_model(&self, row: &PgRow) -> LlmModel {
        use models::ModelMode;

        let capabilities: serde_json::Value = row.get("capabilities");
        let capabilities: Vec<String> = serde_json::from_value(capabilities).unwrap_or_default();

        LlmModel {
            id: row.get("id"),
            provider_id: row.get("provider_id"),
            name: row.get("name"),
            model_id: row.get("model_id"),
            mode: match row.get::<String, _>("mode").as_str() {
                "completion" => ModelMode::Completion,
                "embedding" => ModelMode::Embedding,
                _ => ModelMode::Chat,
            },
            context_window: row.get("context_window"),
            capabilities,
            description: row.try_get("description").ok(),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
        }
    }

    // ============ User Provider Key operations (BYOK) ============

    pub async fn create_user_provider_key(&self, key: &UserProviderKey) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO user_provider_keys (id, user_id, provider_slug, name, api_key_encrypted,
                api_key_prefix, base_url, is_active, priority_level, sort_order, always_use,
                model_filter, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(key.id)
        .bind(key.user_id)
        .bind(&key.provider_slug)
        .bind(&key.name)
        .bind(&key.api_key_encrypted)
        .bind(&key.api_key_prefix)
        .bind(&key.base_url)
        .bind(key.is_active)
        .bind(key.priority_level.as_str())
        .bind(key.sort_order)
        .bind(key.always_use)
        .bind(&key.model_filter)
        .bind(key.created_at)
        .bind(key.updated_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn list_user_provider_keys(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserProviderKey>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM user_provider_keys WHERE user_id = $1 ORDER BY provider_slug, sort_order",
        )
        .bind(user_id)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_user_provider_key(r)).collect())
    }

    pub async fn list_user_provider_keys_by_provider(
        &self,
        user_id: Uuid,
        provider_slug: &str,
    ) -> Result<Vec<UserProviderKey>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM user_provider_keys WHERE user_id = $1 AND provider_slug = $2 AND is_active = TRUE ORDER BY priority_level, sort_order",
        )
        .bind(user_id)
        .bind(provider_slug)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_user_provider_key(r)).collect())
    }

    pub async fn update_user_provider_key(
        &self,
        id: Uuid,
        user_id: Uuid,
        name: Option<&str>,
        is_active: Option<bool>,
        priority_level: Option<&str>,
        sort_order: Option<i32>,
        always_use: Option<bool>,
        model_filter: Option<&str>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE user_provider_keys SET
                name = COALESCE($3, name),
                is_active = COALESCE($4, is_active),
                priority_level = COALESCE($5, priority_level),
                sort_order = COALESCE($6, sort_order),
                always_use = COALESCE($7, always_use),
                model_filter = COALESCE($8, model_filter),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(is_active)
        .bind(priority_level)
        .bind(sort_order)
        .bind(always_use)
        .bind(model_filter)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_user_provider_key(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), DbError> {
        sqlx::query("DELETE FROM user_provider_keys WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    pub async fn list_all_user_provider_keys_with_users(
        &self,
    ) -> Result<Vec<(UserProviderKey, String)>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT upk.*, u.email as user_email
            FROM user_provider_keys upk
            JOIN users u ON upk.user_id = u.id
            ORDER BY upk.created_at DESC
            "#,
        )
        .fetch_all(self.inner())
        .await?;

        Ok(rows
            .iter()
            .map(|r| {
                let upk = self.row_to_user_provider_key(r);
                let user_email: String = r.get("user_email");
                (upk, user_email)
            })
            .collect())
    }

    pub async fn admin_delete_user_provider_key(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM user_provider_keys WHERE id = $1")
            .bind(id)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    fn row_to_user_provider_key(&self, row: &PgRow) -> UserProviderKey {
        UserProviderKey {
            id: row.get("id"),
            user_id: row.get("user_id"),
            provider_slug: row.get("provider_slug"),
            name: row.get("name"),
            api_key_encrypted: row.get("api_key_encrypted"),
            api_key_prefix: row.get("api_key_prefix"),
            base_url: row.get("base_url"),
            is_active: row.get("is_active"),
            priority_level: PriorityLevel::from_str(
                &row.get::<String, _>("priority_level"),
            ),
            sort_order: row.get("sort_order"),
            always_use: row.get("always_use"),
            model_filter: row.get("model_filter"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
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
        .bind(tx.transaction_type.as_str())
        .bind(Decimal::from_f64_retain(tx.amount).unwrap_or_default())
        .bind(&tx.plan)
        .bind(tx.status.as_str())
        .bind(&tx.description)
        .bind(tx.created_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn list_transactions(
        &self,
        user_id: Uuid,
        limit: i32,
    ) -> Result<Vec<Transaction>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_transaction(r)).collect())
    }

    fn row_to_transaction(&self, row: &PgRow) -> Transaction {
        use models::subscription::{TransactionStatus, TransactionType};

        Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            transaction_type: TransactionType::from_str(
                &row.get::<String, _>("transaction_type"),
            ),
            amount: rust_decimal::prelude::ToPrimitive::to_f64(&row.get::<Decimal, _>("amount"))
                .unwrap_or(0.0),
            plan: row.get("plan"),
            status: TransactionStatus::from_str(
                &row.get::<String, _>("status"),
            ),
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
        .bind(log.status.as_str())
        .bind(&log.error_message)
        .bind(log.created_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    /// Get total tokens used by a user in a given period
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
            usage_by_provider: provider_rows
                .iter()
                .map(|r| models::ProviderUsage {
                    provider: r.get("provider_id"),
                    requests: r.get("requests"),
                    input_tokens: r.get("input_tokens"),
                    output_tokens: r.get("output_tokens"),
                })
                .collect(),
            usage_by_model: model_rows
                .iter()
                .map(|r| models::ModelUsage {
                    model: r.get("model_id"),
                    provider: r.get("provider_id"),
                    requests: r.get("requests"),
                    input_tokens: r.get("input_tokens"),
                    output_tokens: r.get("output_tokens"),
                })
                .collect(),
        })
    }

    // ============ Admin operations ============

    pub async fn list_users(
        &self,
        offset: i64,
        limit: i64,
        search: &str,
    ) -> Result<Vec<User>, DbError> {
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
    ) -> Result<(), DbError> {
        if let Some(phone) = phone {
            sqlx::query("UPDATE users SET phone = $2, updated_at = NOW() WHERE id = $1")
                .bind(user_id)
                .bind(phone)
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
        api_type: Option<&str>,
        openai_api_url: Option<&str>,
        anthropic_api_url: Option<&str>,
        is_active: Option<bool>,
        priority: Option<i32>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE providers SET
                name = COALESCE($2, name),
                slug = COALESCE($3, slug),
                api_base_url = COALESCE($4, api_base_url),
                api_type = COALESCE($5, api_type),
                openai_api_url = COALESCE($6, openai_api_url),
                anthropic_api_url = COALESCE($7, anthropic_api_url),
                is_active = COALESCE($8, is_active),
                priority = COALESCE($9, priority)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(api_base_url)
        .bind(api_type)
        .bind(openai_api_url)
        .bind(anthropic_api_url)
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

    pub async fn get_provider_key_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<ProviderKey>, DbError> {
        let row = sqlx::query(
            "SELECT * FROM provider_keys WHERE provider_slug = $1 AND is_active = true",
        )
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
        let rows = sqlx::query("SELECT * FROM models ORDER BY is_active DESC, name")
            .fetch_all(self.inner())
            .await?;

        Ok(rows.iter().map(|r| self.row_to_model(r)).collect())
    }

    pub async fn update_model(
        &self,
        id: Uuid,
        name: Option<&str>,
        model_id: Option<&str>,
        provider_id: Option<&str>,
        context_window: Option<i32>,
        capabilities: Option<serde_json::Value>,
        description: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<(), DbError> {
        sqlx::query(
            r#"
            UPDATE models SET
                name = COALESCE($2, name),
                model_id = COALESCE($3, model_id),
                provider_id = COALESCE($4, provider_id),
                context_window = COALESCE($5, context_window),
                capabilities = COALESCE($6, capabilities),
                description = COALESCE($8, description),
                is_active = COALESCE($7, is_active)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(model_id)
        .bind(provider_id)
        .bind(context_window)
        .bind(capabilities)
        .bind(is_active)
        .bind(description)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_model(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM models WHERE id = $1")
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

        Ok(rows
            .iter()
            .map(|r| {
                let tx = self.row_to_transaction(r);
                let email: String = r.get("user_email");
                (tx, email)
            })
            .collect())
    }

    pub async fn count_all_transactions(
        &self,
        tx_type: &str,
        status: &str,
    ) -> Result<i64, DbError> {
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

        let total_revenue: f64 = sqlx::query(
            r#"SELECT COALESCE(SUM(amount), 0) as total FROM transactions WHERE status = 'completed' AND amount > 0"#
        )
        .fetch_one(self.inner())
        .await?
        .get::<rust_decimal::Decimal, _>("total")
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);

        let api_calls_today: i64 =
            sqlx::query("SELECT COUNT(*) as count FROM api_logs WHERE created_at >= CURRENT_DATE")
                .fetch_one(self.inner())
                .await?
                .get("count");

        Ok(DashboardStats {
            total_users: user_count,
            total_revenue,
            api_calls_today,
        })
    }
}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_revenue: f64,
    pub api_calls_today: i64,
}

/// Revenue trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueTrend {
    pub label: String,
    pub value: f64,
    pub date: String,
}

/// Recent activity item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub user_email: String,
    pub action_type: String,
    pub description: String,
    pub time_ago: String,
}

impl PostgresPool {
    pub async fn get_revenue_trends(&self, days: i32) -> Result<Vec<RevenueTrend>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                TO_CHAR(day, 'Dy') AS label,
                COALESCE(SUM(t.amount), 0)::FLOAT8 AS value,
                TO_CHAR(day, 'YYYY-MM-DD') AS date
            FROM generate_series(
                CURRENT_DATE - ($1::int - 1) * INTERVAL '1 day',
                CURRENT_DATE,
                INTERVAL '1 day'
            ) AS day
            LEFT JOIN transactions t
                ON t.created_at::date = day::date
                AND t.status = 'completed'
                AND t.amount > 0
            GROUP BY day
            ORDER BY day
            "#,
        )
        .bind(days)
        .fetch_all(self.inner())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| RevenueTrend {
                label: row.get("label"),
                value: row.get::<f64, _>("value"),
                date: row.get("date"),
            })
            .collect())
    }

    pub async fn get_recent_activity(&self, limit: i64) -> Result<Vec<RecentActivity>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM (
                SELECT
                    u.email AS user_email,
                    t.transaction_type AS action_type,
                    CASE
                        WHEN t.transaction_type = 'token_purchase' THEN 'Purchased credits'
                        WHEN t.transaction_type = 'refund' THEN 'Refund processed'
                        ELSE t.transaction_type
                    END AS description,
                    EXTRACT(EPOCH FROM (NOW() - t.created_at))::int AS seconds_ago
                FROM transactions t
                JOIN users u ON u.id = t.user_id

                UNION ALL

                SELECT
                    u.email AS user_email,
                    'api_call' AS action_type,
                    'API call via ' || al.provider_id || ' (' || al.model_id || ')' AS description,
                    EXTRACT(EPOCH FROM (NOW() - al.created_at))::int AS seconds_ago
                FROM api_logs al
                JOIN users u ON u.id = al.user_id
            ) combined
            ORDER BY seconds_ago ASC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(self.inner())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let seconds: i32 = row.get("seconds_ago");
                let time_ago = if seconds < 60 {
                    format!("{}s", seconds)
                } else if seconds < 3600 {
                    format!("{}m", seconds / 60)
                } else if seconds < 86400 {
                    format!("{}h", seconds / 3600)
                } else {
                    format!("{}d", seconds / 86400)
                };
                RecentActivity {
                    user_email: row.get("user_email"),
                    action_type: row.get("action_type"),
                    description: row.get("description"),
                    time_ago,
                }
            })
            .collect())
    }

    // ============ Model Pricing operations ============

    pub async fn list_model_pricing(&self) -> Result<Vec<ModelPricing>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM model_pricing WHERE is_active = TRUE ORDER BY provider_slug, model_slug",
        )
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_model_pricing(r)).collect())
    }

    pub async fn get_model_pricing(&self, model_slug: &str) -> Result<Option<ModelPricing>, DbError> {
        let row: Option<PgRow> = sqlx::query(
            "SELECT * FROM model_pricing WHERE model_slug = $1 AND is_active = TRUE AND effective_from <= NOW() AND (effective_until IS NULL OR effective_until > NOW())",
        )
        .bind(model_slug)
        .fetch_optional(self.inner())
        .await?;

        Ok(row.map(|r| self.row_to_model_pricing(&r)))
    }

    pub async fn upsert_model_pricing(&self, p: &ModelPricing) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO model_pricing (id, model_slug, provider_slug, prompt_price, completion_price,
                image_price, reasoning_price, cache_read_price, request_price, pricing_mode,
                avg_tokens_per_request, effective_from, effective_until, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (model_slug) DO UPDATE SET
                provider_slug = $3, prompt_price = $4, completion_price = $5, image_price = $6,
                reasoning_price = $7, cache_read_price = $8, request_price = $9, pricing_mode = $10,
                avg_tokens_per_request = $11, effective_from = $12, effective_until = $13,
                is_active = $14, updated_at = $16
            "#,
        )
        .bind(p.id)
        .bind(&p.model_slug)
        .bind(&p.provider_slug)
        .bind(p.prompt_price)
        .bind(p.completion_price)
        .bind(p.image_price)
        .bind(p.reasoning_price)
        .bind(p.cache_read_price)
        .bind(p.request_price)
        .bind(p.pricing_mode.as_str())
        .bind(p.avg_tokens_per_request)
        .bind(p.effective_from)
        .bind(p.effective_until)
        .bind(p.is_active)
        .bind(p.created_at)
        .bind(p.updated_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_model_pricing(&self, model_slug: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM model_pricing WHERE model_slug = $1")
            .bind(model_slug)
            .execute(self.inner())
            .await?;
        Ok(())
    }

    fn row_to_model_pricing(&self, row: &PgRow) -> ModelPricing {
        ModelPricing {
            id: row.get("id"),
            model_slug: row.get("model_slug"),
            provider_slug: row.get("provider_slug"),
            prompt_price: row.get("prompt_price"),
            completion_price: row.get("completion_price"),
            image_price: row.get("image_price"),
            reasoning_price: row.get("reasoning_price"),
            cache_read_price: row.get("cache_read_price"),
            request_price: row.get("request_price"),
            pricing_mode: PricingMode::from_str(
                &row.get::<String, _>("pricing_mode"),
            ),
            avg_tokens_per_request: row.get("avg_tokens_per_request"),
            effective_from: row.get("effective_from"),
            effective_until: row.get("effective_until"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    // ============ User Balance operations ============

    pub async fn get_user_balance(&self, user_id: Uuid) -> Result<UserBalance, DbError> {
        let row: Option<PgRow> = sqlx::query("SELECT * FROM user_balances WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(self.inner())
            .await?;

        match row {
            Some(r) => Ok(self.row_to_user_balance(&r)),
            None => Ok(UserBalance::new(user_id)),
        }
    }

    pub async fn ensure_user_balance(&self, user_id: Uuid) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO user_balances (user_id, balance, total_purchased, total_consumed, updated_at)
            VALUES ($1, 0, 0, 0, NOW())
            ON CONFLICT (user_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(self.inner())
        .await?;
        Ok(())
    }

    pub async fn deduct_balance(
        &self,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<(), DbError> {
        let result = sqlx::query(
            r#"
            UPDATE user_balances SET
                balance = balance - $2,
                total_consumed = total_consumed + $2,
                updated_at = NOW()
            WHERE user_id = $1 AND balance >= $2
            "#,
        )
        .bind(user_id)
        .bind(amount)
        .execute(self.inner())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::InsufficientBalance);
        }
        Ok(())
    }

    pub async fn add_balance(
        &self,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<(), DbError> {
        self.ensure_user_balance(user_id).await?;
        sqlx::query(
            r#"
            UPDATE user_balances SET
                balance = balance + $2,
                total_purchased = total_purchased + $2,
                updated_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .bind(amount)
        .execute(self.inner())
        .await?;
        Ok(())
    }

    fn row_to_user_balance(&self, row: &PgRow) -> UserBalance {
        UserBalance {
            user_id: row.get("user_id"),
            balance: row.get("balance"),
            total_purchased: row.get("total_purchased"),
            total_consumed: row.get("total_consumed"),
            auto_topup_threshold: row.get("auto_topup_threshold"),
            auto_topup_amount: row.get("auto_topup_amount"),
            updated_at: row.get("updated_at"),
        }
    }

    // ============ Token Charge operations ============

    pub async fn create_token_charge(&self, charge: &TokenCharge) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO token_charges (id, user_id, api_log_id, generation_id, key_source,
                user_provider_key_id, provider_key_id, model_slug, provider_slug,
                input_tokens, output_tokens, reasoning_tokens, image_count, cache_read_tokens,
                prompt_cost, completion_cost, image_cost, reasoning_cost, cache_read_cost,
                request_cost, total_cost, is_free, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19, $20, $21, $22, $23)
            "#,
        )
        .bind(charge.id)
        .bind(charge.user_id)
        .bind(charge.api_log_id)
        .bind(charge.generation_id)
        .bind(&charge.key_source)
        .bind(charge.user_provider_key_id)
        .bind(charge.provider_key_id)
        .bind(&charge.model_slug)
        .bind(&charge.provider_slug)
        .bind(charge.input_tokens)
        .bind(charge.output_tokens)
        .bind(charge.reasoning_tokens)
        .bind(charge.image_count)
        .bind(charge.cache_read_tokens)
        .bind(charge.prompt_cost)
        .bind(charge.completion_cost)
        .bind(charge.image_cost)
        .bind(charge.reasoning_cost)
        .bind(charge.cache_read_cost)
        .bind(charge.request_cost)
        .bind(charge.total_cost)
        .bind(charge.is_free)
        .bind(charge.created_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn list_token_charges(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TokenCharge>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM token_charges WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| self.row_to_token_charge(r)).collect())
    }

    pub async fn get_daily_charges(
        &self,
        user_id: Uuid,
        days: i32,
    ) -> Result<Vec<models::DailyCharge>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE(created_at) as day,
                COALESCE(SUM(input_tokens), 0)::bigint as input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint as output_tokens,
                COALESCE(SUM(total_cost), 0) as total_cost
            FROM token_charges
            WHERE user_id = $1
              AND created_at >= NOW() - ($2 || ' days')::interval
            GROUP BY DATE(created_at)
            ORDER BY day ASC
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(self.inner())
        .await?;

        Ok(rows
            .iter()
            .map(|r| models::DailyCharge {
                day: r.get::<chrono::NaiveDate, _>("day"),
                input_tokens: r.get::<i64, _>("input_tokens") as i64,
                output_tokens: r.get::<i64, _>("output_tokens") as i64,
                total_cost: r.get::<rust_decimal::Decimal, _>("total_cost"),
            })
            .collect())
    }

    pub async fn get_daily_charges_by_model(
        &self,
        user_id: Uuid,
        days: i32,
    ) -> Result<Vec<models::DailyModelCharge>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE(created_at) as day,
                model_slug,
                COALESCE(SUM(total_cost), 0) as total_cost
            FROM token_charges
            WHERE user_id = $1
              AND created_at >= NOW() - ($2 || ' days')::interval
            GROUP BY DATE(created_at), model_slug
            ORDER BY day ASC, model_slug ASC
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(self.inner())
        .await?;

        Ok(rows
            .iter()
            .map(|r| models::DailyModelCharge {
                day: r.get::<chrono::NaiveDate, _>("day"),
                model_slug: r.get::<String, _>("model_slug"),
                total_cost: r.get::<rust_decimal::Decimal, _>("total_cost"),
            })
            .collect())
    }

    fn row_to_token_charge(&self, row: &PgRow) -> TokenCharge {
        TokenCharge {
            id: row.get("id"),
            user_id: row.get("user_id"),
            api_log_id: row.get("api_log_id"),
            generation_id: row.get("generation_id"),
            key_source: row.get("key_source"),
            user_provider_key_id: row.get("user_provider_key_id"),
            provider_key_id: row.get("provider_key_id"),
            model_slug: row.get("model_slug"),
            provider_slug: row.get("provider_slug"),
            input_tokens: row.get("input_tokens"),
            output_tokens: row.get("output_tokens"),
            reasoning_tokens: row.get("reasoning_tokens"),
            image_count: row.get("image_count"),
            cache_read_tokens: row.get("cache_read_tokens"),
            prompt_cost: row.get("prompt_cost"),
            completion_cost: row.get("completion_cost"),
            image_cost: row.get("image_cost"),
            reasoning_cost: row.get("reasoning_cost"),
            cache_read_cost: row.get("cache_read_cost"),
            request_cost: row.get("request_cost"),
            total_cost: row.get("total_cost"),
            is_free: row.get("is_free"),
            created_at: row.get("created_at"),
        }
    }

    // ============ Token Package operations ============

    pub async fn list_token_packages(&self) -> Result<Vec<TokenPackage>, DbError> {
        let rows = sqlx::query(
            "SELECT * FROM token_packages WHERE is_active = TRUE ORDER BY sort_order",
        )
        .fetch_all(self.inner())
        .await?;

        Ok(rows.iter().map(|r| TokenPackage {
            id: r.get("id"),
            name: r.get("name"),
            credits: r.get("credits"),
            price: r.get("price"),
            currency: r.get("currency"),
            bonus_credits: r.get("bonus_credits"),
            is_active: r.get("is_active"),
            sort_order: r.get("sort_order"),
            created_at: r.get("created_at"),
        }).collect())
    }

    pub async fn upsert_token_package(&self, pkg: &TokenPackage) -> Result<(), DbError> {
        sqlx::query(
            r#"
            INSERT INTO token_packages (id, name, credits, price, currency, bonus_credits,
                is_active, sort_order, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                name = $2, credits = $3, price = $4, currency = $5, bonus_credits = $6,
                is_active = $7, sort_order = $8
            "#,
        )
        .bind(pkg.id)
        .bind(&pkg.name)
        .bind(pkg.credits)
        .bind(pkg.price)
        .bind(&pkg.currency)
        .bind(pkg.bonus_credits)
        .bind(pkg.is_active)
        .bind(pkg.sort_order)
        .bind(pkg.created_at)
        .execute(self.inner())
        .await?;

        Ok(())
    }

    pub async fn delete_token_package(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM token_packages WHERE id = $1")
            .bind(id)
            .execute(self.inner())
            .await?;
        Ok(())
    }
}
