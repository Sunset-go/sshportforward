//! 隧道启动/关闭、连接状态机与私钥选择命令。

use std::collections::HashMap;

use tauri::{AppHandle, Emitter, State};

use crate::commands::settings::CLOSE_TO_TRAY_KEY;
use crate::db::{self, AppConnState, AppDb};
use crate::models::TrafficStat;
use crate::ssh::{self, SshConfig, TunnelState};

use super::hosts::{get_host_row, load_rules};

/// 查询当前连接状态（`idle` / `connecting` / `connected` / `disconnecting` / `error`）
#[tauri::command]
pub fn get_conn_state(conn: State<'_, AppConnState>) -> String {
    conn.0
        .read()
        .expect("连接状态锁已中毒")
        .clone()
}

fn set_state(conn: &AppConnState, state: &str) {
    *conn.0.write().expect("连接状态锁已中毒") = state.to_string();
}

/// 启动隧道：从加密库读取主机凭据并建立真实 SSH 会话，随后为启用规则建立转发
#[tauri::command]
pub async fn start_tunnel(
    app: AppHandle,
    db: State<'_, AppDb>,
    conn: State<'_, AppConnState>,
    tunnel: State<'_, TunnelState>,
    host_id: String,
) -> Result<(), String> {
    set_state(&conn, "connecting");

    let row = get_host_row(&db, &host_id).await?;
    let config = SshConfig {
        host: row.host.clone(),
        port: row.port as u16,
        username: row.username.clone(),
        password: crate::crypto::decrypt(&row.password)?,
        key_path: crate::crypto::decrypt(&row.key_path)?,
    };
    let rules = load_rules(&db.0, &host_id).await?;

    match ssh::start(config, rules).await {
        Ok((session, traffic)) => {
            *tunnel.session.lock().expect("隧道会话锁已中毒") = Some(session);
            *tunnel.traffic.lock().expect("隧道流量锁已中毒") = traffic;
            set_state(&conn, "connected");

            // 隧道已运行但关闭行为未设为最小化到托盘时，提醒用户开启，
            // 避免误点关闭窗口导致隧道中断
            let pref = db::get_setting(&db.0, CLOSE_TO_TRAY_KEY)
                .await
                .ok()
                .flatten();
            if pref.as_deref() != Some("tray") {
                let _ = app.emit("ask-start-tray", ());
            }
            Ok(())
        }
        Err(e) => {
            set_state(&conn, "error");
            Err(e)
        }
    }
}

/// 查询当前隧道各规则的上传/下载流量（字节），规则 id 为键
#[tauri::command]
pub fn get_traffic(tunnel: State<'_, TunnelState>) -> Result<HashMap<String, TrafficStat>, String> {
    let map = tunnel.traffic.lock().expect("隧道流量锁已中毒");
    Ok(map
        .iter()
        .map(|(id, cell)| {
            let (up, down) = cell.snapshot();
            (id.clone(), TrafficStat { up, down })
        })
        .collect())
}

/// 关闭隧道：通知转发任务退出并关闭 SSH 会话
#[tauri::command]
pub async fn stop_tunnel(
    conn: State<'_, AppConnState>,
    tunnel: State<'_, TunnelState>,
) -> Result<(), String> {
    set_state(&conn, "disconnecting");
    if let Some(session) = tunnel.session.lock().expect("隧道会话锁已中毒").take() {
        session.shutdown();
    }
    set_state(&conn, "idle");
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
