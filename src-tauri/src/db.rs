//! 数据库连接初始化与迁移。
//!
//! 使用 sqlx 管理 SQLite（文件位于应用数据目录 ssh_tunnel.db），
//! 启动时执行 `db/migrations/001_init.sql` 中的建表语句。

use std::sync::RwLock;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

/// Tauri 共享状态：SQLite 连接池
#[derive(Clone)]
pub struct AppDb(pub SqlitePool);

/// Tauri 共享状态：连接状态机
///
/// 取值：`idle` / `connecting` / `connected` / `disconnecting` / `error`
pub struct AppConnState(pub RwLock<String>);

const MIGRATION_SQL: &str = include_str!("../db/migrations/001_init.sql");

/// 打开应用数据目录下的 SQLite 库并执行迁移，返回连接池。
pub async fn init_db(app: &AppHandle) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    let db_path = dir.join("ssh_tunnel.db");

    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        // 确保 `ON DELETE CASCADE`（删除主机时级联删除规则）生效
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;
    run_migration_sql(&pool).await?;
    Ok(pool)
}

/// 执行 `db/migrations/001_init.sql`：按分号分割逐条执行，
/// 兼容 SQLite 对多语句 prepared statement 的限制。
async fn run_migration_sql(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for stmt in MIGRATION_SQL.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}

/// 统一格式化数据库错误信息
pub fn db_err(e: sqlx::Error, ctx: &str) -> String {
    format!("{ctx}: {e}")
}

/// 生成全局唯一 id（时间戳 + 单调自增，前缀 `host-` / `r-`，匹配前端 mock 的字符串 id 风格）
pub fn gen_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    format!("{prefix}-{millis:x}{n:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_sql_runs_and_cascade_deletes_rules() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let opts = SqliteConnectOptions::new()
                .filename(":memory:")
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(opts)
                .await
                .unwrap();

            // 迁移 SQL 整体可执行
            run_migration_sql(&pool).await.unwrap();

            // 字符串主键可插入
            sqlx::query(
                "INSERT INTO hosts (id, name, host, port, username, password, key_path) VALUES ('host-pc', '办公电脑', '192.168.1.10', 22, 'root', '', '')",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO forward_rules (id, host_id, rule_type, enabled, listen_addr, target_addr) VALUES ('r-1', 'host-pc', 'local', 1, '127.0.0.1:8080', 'localhost:80')",
            )
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query("INSERT INTO logs (level, message) VALUES ('INFO', 'hello')")
                .execute(&pool)
                .await
                .unwrap();

            // 删除主机 → 级联删除规则
            sqlx::query("DELETE FROM hosts WHERE id = 'host-pc'")
                .execute(&pool)
                .await
                .unwrap();
            let rules: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM forward_rules WHERE host_id = 'host-pc'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(rules, 0);

            // 日志不受主机删除影响
            let logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM logs")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(logs, 1);
        });
    }
}