//! 日志读写命令。

use tauri::State;

use crate::db::{db_err, AppDb};
use crate::models::{LogEntryOut, LogRow};

const SELECT_LOGS_SQL: &str = "SELECT id, level, message, created_at FROM logs ORDER BY id ASC";

/// 追加一条日志，返回完整条目
#[tauri::command]
pub async fn add_log(
    db: State<'_, AppDb>,
    level: String,
    message: String,
) -> Result<LogEntryOut, String> {
    sqlx::query("INSERT INTO logs (level, message) VALUES (?, ?)")
        .bind(&level)
        .bind(&message)
        .execute(&db.0)
        .await
        .map_err(|e| db_err(e, "写入日志失败"))?;
    let row = sqlx::query_as::<_, LogRow>(
        "SELECT id, level, message, created_at FROM logs ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&db.0)
    .await
    .map_err(|e| db_err(e, "读取日志失败"))?;
    Ok(row.into_out())
}

/// 拉取日志：传 `limit` 时返回最近 N 条，否则返回全部；均按时间升序
#[tauri::command]
pub async fn list_logs(
    db: State<'_, AppDb>,
    limit: Option<i64>,
) -> Result<Vec<LogEntryOut>, String> {
    let rows = if let Some(lim) = limit {
        sqlx::query_as::<_, LogRow>(
            "SELECT id, level, message, created_at FROM (SELECT * FROM logs ORDER BY id DESC LIMIT ?) ORDER BY id ASC",
        )
        .bind(lim)
        .fetch_all(&db.0)
        .await
        .map_err(|e| db_err(e, "读取日志失败"))?
    } else {
        sqlx::query_as::<_, LogRow>(SELECT_LOGS_SQL)
            .fetch_all(&db.0)
            .await
            .map_err(|e| db_err(e, "读取日志失败"))?
    };
    Ok(rows.into_iter().map(LogRow::into_out).collect())
}

/// 清空全部日志
#[tauri::command]
pub async fn clear_logs(db: State<'_, AppDb>) -> Result<bool, String> {
    sqlx::query("DELETE FROM logs")
        .execute(&db.0)
        .await
        .map_err(|e| db_err(e, "清空日志失败"))?;
    Ok(true)
}

/// 读取日志为 `"[HH:mm:ss] LEVEL message"` 字符串数组（兼容前端 `invoke<string[]>`）
#[tauri::command]
pub async fn read_logs(db: State<'_, AppDb>) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, LogRow>(SELECT_LOGS_SQL)
        .fetch_all(&db.0)
        .await
        .map_err(|e| db_err(e, "读取日志失败"))?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let e = r.into_out();
            format!("[{}] {} {}", e.time, e.level, e.message)
        })
        .collect())
}