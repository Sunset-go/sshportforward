//! 隧道启动/关闭、连接状态机与私钥选择命令。
//!
//! 当前为 mock 实现：模拟握手耗时与状态流转，不建立真实 SSH 通道。

use std::time::Duration;

use tauri::State;

use crate::db::{db_err, AppConnState, AppDb};

use super::hosts::load_rules;

/// 查询当前连接状态（`idle` / `connecting` / `connected` / `disconnecting` / `error`）
#[tauri::command]
pub fn get_conn_state(conn: State<'_, AppConnState>) -> String {
    conn.0.read().unwrap().clone()
}

/// 手动设置连接状态（通常由前端状态机驱动）
#[tauri::command]
pub fn set_conn_state(conn: State<'_, AppConnState>, state: String) -> bool {
    *conn.0.write().unwrap() = state;
    true
}

async fn write_log(pool: &sqlx::SqlitePool, level: &str, message: &str) -> Result<(), String> {
    sqlx::query("INSERT INTO logs (level, message) VALUES (?, ?)")
        .bind(level)
        .bind(message)
        .execute(pool)
        .await
        .map_err(|e| db_err(e, "写入日志失败"))?;
    Ok(())
}

/// 启动隧道（mock）：握手 → 认证 → 成功，并对主机的启用规则逐个写 established 日志
#[tauri::command]
pub async fn start_tunnel(
    db: State<'_, AppDb>,
    conn: State<'_, AppConnState>,
    host_id: String,
    host: String,
    port: i64,
    username: String,
) -> Result<(), String> {
    {
        let mut s = conn.0.write().unwrap();
        *s = String::from("connecting");
    }
    write_log(&db.0, "INFO", &format!("连接 {host}:{port}（用户 {username}）")).await?;
    tokio::time::sleep(Duration::from_millis(1500)).await;

    {
        let mut s = conn.0.write().unwrap();
        *s = String::from("connected");
    }
    write_log(&db.0, "SUCCESS", &format!("已连接 {host}:{port}")).await?;

    let rules = load_rules(&db.0, &host_id).await?;
    for rule in rules.iter().filter(|r| r.enabled) {
        let flag = match rule.rule_type.as_str() {
            "local" => "-L",
            "remote" => "-R",
            _ => "-D",
        };
        let arrow = if rule.target_addr.is_empty() {
            String::new()
        } else {
            format!(" → {}", rule.target_addr)
        };
        write_log(
            &db.0,
            "SUCCESS",
            &format!("已建立转发 [{flag}] {}{arrow}", rule.listen_addr),
        )
        .await?;
    }
    Ok(())
}

/// 关闭隧道（mock）
#[tauri::command]
pub async fn stop_tunnel(
    db: State<'_, AppDb>,
    conn: State<'_, AppConnState>,
    host_id: String,
) -> Result<(), String> {
    {
        let mut s = conn.0.write().unwrap();
        *s = String::from("disconnecting");
    }
    write_log(&db.0, "INFO", "正在关闭隧道…").await?;
    tokio::time::sleep(Duration::from_millis(800)).await;

    {
        let mut s = conn.0.write().unwrap();
        *s = String::from("idle");
    }
    write_log(&db.0, "SUCCESS", &format!("主机 {host_id} 的隧道已关闭")).await?;
    Ok(())
}

/// 弹出系统文件选择器选择 SSH 私钥；用户取消时返回空字符串
#[tauri::command]
pub fn pick_key_file(app: tauri::AppHandle) -> String {
    use tauri_plugin_dialog::DialogExt;

    app.dialog()
        .file()
        .set_title("选择 SSH 私钥")
        .add_filter("SSH Keys", &["pem", "key", "ed25519", "rsa"])
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok())
        .map(|pb| pb.to_string_lossy().to_string())
        .unwrap_or_default()
}