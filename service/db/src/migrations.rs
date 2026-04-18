//! 数据库迁移模块
//!
//! 服务启动时自动执行待执行的迁移脚本
//! 通过 `schema_migrations` 表跟踪已执行的迁移

use std::path::Path;
use sqlx::{PgPool, Row};
use tracing::{info, warn, error};

/// 执行所有待执行的数据库迁移
///
/// # 参数
/// * `pool` - PostgreSQL 连接池
/// * `migrations_path` - 迁移文件目录路径
///
/// 迁移文件命名格式: `001_description.sql`
pub async fn run_migrations(
    pool: &PgPool,
    migrations_path: &str,
) -> anyhow::Result<()> {
    // 确保 migrations 表存在
    ensure_migrations_table(pool).await?;

    // 获取已执行的迁移
    let executed: Vec<String> = sqlx::query("SELECT version FROM schema_migrations")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|row| row.get::<String, _>("version"))
        .collect();

    info!("Found {} migrations already executed", executed.len());

    // 读取迁移目录
    let path = Path::new(migrations_path);
    if !path.exists() {
        warn!("Migrations directory not found: {}", migrations_path);
        return Ok(());
    }

    // 收集所有迁移文件
    let mut migrations: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "sql")
        })
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            // 跳过 seed 文件（只在初始化时执行一次）
            !name.contains("_seed_") && !name.contains("seed_")
        })
        .collect();

    // 按文件名排序（确保执行顺序）
    migrations.sort_by(|a, b| {
        a.file_name().cmp(&b.file_name())
    });

    // 执行待执行的迁移
    for entry in migrations {
        let filename = entry.file_name().to_string_lossy().to_string();

        // 提取版本号（文件名格式: 001_name.sql）
        let version = filename.split('_').next().unwrap_or(&filename).to_string();

        if executed.contains(&version) {
            continue;
        }

        // 读取迁移 SQL
        let sql = std::fs::read_to_string(entry.path())?;

        info!("Running migration: {} ({})", version, filename);

        // 在事务中执行迁移
        let mut tx = pool.begin().await?;

        // 分割多条 SQL 语句并逐条执行
        let statements: Vec<&str> = sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut exec_error = None;
        for stmt in &statements {
            if let Err(e) = sqlx::query(stmt).execute(&mut *tx).await {
                exec_error = Some(e);
                break;
            }
        }

        if let Some(e) = exec_error {
            let err_msg = format!("Migration {} failed: {}", version, e);
            error!("{}", err_msg);
            // 如果是 "already exists" 错误，记录为已执行
            if err_msg.contains("already exists") {
                sqlx::query("INSERT INTO schema_migrations (version, name) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                    .bind(&version)
                    .bind(&filename)
                    .execute(pool)
                    .await?;
                info!("Migration {} already exists, recorded as executed", version);
            } else {
                tx.rollback().await?;
                return Err(anyhow::anyhow!(err_msg));
            }
        } else {
            // 记录迁移
            sqlx::query("INSERT INTO schema_migrations (version, name) VALUES ($1, $2)")
                .bind(&version)
                .bind(&filename)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            info!("Migration {} completed successfully", version);
        }
    }

    info!("All migrations completed");
    Ok(())
}

/// 确保 schema_migrations 表存在
async fn ensure_migrations_table(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
