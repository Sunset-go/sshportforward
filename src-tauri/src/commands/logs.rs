//! 日志写命令（前端为唯一日志源，DB 仅作持久化镜像）。

use tauri::State;

use crate::db::{db_err, AppDb};
use crate::models::{LogEntryOut, LogRow};

/// 追加一条日志，用 `RETURNING` 单次查询返回完整条目
#[tauri::command]
pub async fn add_log(
    db: State<'_, AppDb>,
    level: String,
    message: String,
) -> Result<LogEntryOut, String> {
    let row = sqlx::query_as::<_, LogRow>(
        "INSERT INTO logs (level, message) VALUES (?, ?) RETURNING id, level, message, created_at",
    )
    .bind(&level)
    .bind(&message)
    .fetch_one(&db.0)
    .await
    .map_err(|e| db_err(e, "写入日志失败"))?;
    Ok(row.into_out())
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
