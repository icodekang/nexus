use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, MissedTickBehavior};

use db::PostgresPool;

const CHECK_INTERVAL_SECS: u64 = 300;

pub async fn start_session_health_checker(
    db: Arc<PostgresPool>,
    account_pool: std::sync::Arc<provider_client::AccountPool>,
) {
    tracing::info!("Session health checker started (interval: {}s)", CHECK_INTERVAL_SECS);

    let mut ticker = interval(Duration::from_secs(CHECK_INTERVAL_SECS));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        let _ = ticker.tick().await;

        if let Err(e) = check_expired_sessions(&db, &account_pool).await {
            tracing::error!("Session health check failed: {}", e);
        }
    }
}

async fn check_expired_sessions(
    db: &Arc<PostgresPool>,
    account_pool: &std::sync::Arc<provider_client::AccountPool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let accounts = db.list_browser_accounts().await?;

    for account in accounts {
        if account.session_status != "active" {
            continue;
        }

        let Some(expires_at) = account.session_expires_at else {
            continue;
        };

        let now = chrono::Utc::now();
        let days_until_expiry = (expires_at - now).num_days();

        if days_until_expiry < 7 {
            tracing::info!("Account {} session expires in {} days", account.id, days_until_expiry);
        }

        if days_until_expiry < 0 {
            tracing::warn!("Account {} session expired", account.id);
            db.update_browser_account_status(account.id, models::BrowserAccountStatus::Expired).await?;
            account_pool.unregister_account(account.id).await;
        }
    }

    Ok(())
}
