//! 应用设置读写、窗口隐藏（收容到托盘）与退出命令。

use tauri::{AppHandle, Manager, State};

use crate::db::AppDb;

/// 设置键：关闭窗口时的行为（`tray` = 最小化到托盘，`exit` = 直接退出）。
/// 未设置时关闭窗口会向前端发送询问事件。
pub const CLOSE_TO_TRAY_KEY: &str = "close_to_tray";

/// 读取应用设置，键不存在返回 None
#[tauri::command]
pub async fn get_app_setting(db: State<'_, AppDb>, key: String) -> Result<Option<String>, String> {
    crate::db::get_setting(&db.0, &key)
        .await
        .map_err(|e| crate::db::db_err(e, "读取设置失败"))
}

/// 写入应用设置（存在则覆盖）
#[tauri::command]
pub async fn set_app_setting(db: State<'_, AppDb>, key: String, value: String) -> Result<(), String> {
    crate::db::set_setting(&db.0, &key, &value)
        .await
        .map_err(|e| crate::db::db_err(e, "写入设置失败"))
}

/// 隐藏主窗口（收容到系统托盘）
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}

/// 真正退出应用（绕过托盘隐藏逻辑）
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// 按前端当前语言更新托盘菜单文案（启动与切换语言时调用）
#[tauri::command]
pub fn set_tray_texts(
    app: AppHandle,
    show: String,
    close_to_tray: String,
    quit: String,
) -> Result<(), String> {
    let menu = app.state::<crate::TrayMenuState>().inner();
    menu.show
        .set_text(show)
        .map_err(|e| format!("更新托盘菜单失败: {e}"))?;
    menu.close_to_tray
        .set_text(close_to_tray)
        .map_err(|e| format!("更新托盘菜单失败: {e}"))?;
    menu.quit
        .set_text(quit)
        .map_err(|e| format!("更新托盘菜单失败: {e}"))?;
    Ok(())
}
