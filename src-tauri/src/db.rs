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

/// 迁移列表：`(版本号, SQL)`。版本号为纯数字字符串，按顺序递增；
/// 新增迁移时在此追加即可，启动时仅执行大于当前 `PRAGMA user_version` 的项。
const MIGRATIONS: &[(&str, &str)] = &[
    ("1", include_str!("../db/migrations/001_init.sql")),
    ("2", include_str!("../db/migrations/002_settings.sql")),
];

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
    run_migrations(&pool).await?;
    Ok(pool)
}

/// 按 `PRAGMA user_version` 版本化执行迁移，只跑版本号更大的迁移。
async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let current: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await?;

    for (version, sql) in MIGRATIONS {
        let ver: i64 = version
            .parse()
            .map_err(|_| sqlx::Error::Protocol("非法迁移版本号".into()))?;
        if ver <= current {
            continue;
        }
        for stmt in split_sql(sql) {
            sqlx::query(stmt).execute(pool).await?;
        }
        // 逐版本写回，保证迁移失败时版本号不虚增
        sqlx::query(&format!("PRAGMA user_version = {ver}"))
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// 分号感知的 SQL 拆分：忽略单引号字符串字面量与 `--` 行注释内的分号。
/// 兼容 SQLite 对多语句 prepared statement 的限制。
fn split_sql(sql: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut in_line_comment = false;
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_string {
            if c == '\'' {
                // SQL 字符串内转义的单引号 '' 跳过
                if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    i += 2;
                    continue;
                }
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '\'' => in_string = true,
            '-' if i + 1 < bytes.len() && bytes[i + 1] == b'-' => {
                in_line_comment = true;
                i += 1;
            }
            ';' => {
                let stmt = sql[start..i].trim();
                if !stmt.is_empty() {
                    out.push(stmt);
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let tail = sql[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// 统一格式化数据库错误信息
pub fn db_err(e: sqlx::Error, ctx: &str) -> String {
    format!("{ctx}: {e}")
}

/// 读取应用设置（app_settings 表），键不存在时返回 None。
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let value: Option<(String,)> = sqlx::query_as("SELECT value FROM app_settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(value.map(|(v,)| v))
}

/// 写入应用设置（存在则覆盖）。
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
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
    fn split_sql_ignores_semicolons_in_strings_and_comments() {
        let sql = "CREATE TABLE a (x TEXT DEFAULT 'a;b'); -- comment; here\nINSERT INTO a (x) VALUES ('1');";
        let stmts = split_sql(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("'a;b'"));
        assert!(stmts[1].contains("INSERT"));
    }

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
            run_migrations(&pool).await.unwrap();

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