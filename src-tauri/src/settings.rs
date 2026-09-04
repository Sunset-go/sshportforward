//! 应用偏好持久化（settings.json）：主题、语言、开机自启。
//!
//! `minimize_to_tray` 不落 JSON，而是桥接到现有 SQLite `app_settings` 表的
//! `close_to_tray` 键，使 `lib.rs` 关闭拦截、托盘菜单勾选、CloseBehaviorDialog
//! 继续以 DB 为单一真源工作。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::commands::settings::CLOSE_TO_TRAY_KEY;
use crate::db::{self, AppDb};

/// 应用偏好。序列化 camelCase 以匹配前端 `useSettings` 的字段名。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub locale: String,
    /// 不持久化到 settings.json：由 DB `close_to_tray` 键决定，避免双重真源。
    #[serde(skip)]
    pub minimize_to_tray: bool,
    pub auto_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            locale: "zh".into(),
            minimize_to_tray: false,
            auto_start: false,
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取 app_data_dir 失败: {e}"))?;
    Ok(dir.join("settings.json"))
}

/// 读取偏好：先读 settings.json（theme/locale/autoStart），再从 DB `close_to_tray`
/// 覆盖 `minimizeToTray`，保证与现有关闭逻辑一致。
#[tauri::command]
pub async fn get_settings(app: AppHandle, db: State<'_, AppDb>) -> Result<Settings, String> {
    let mut settings = match settings_path(&app) {
        Ok(path) => match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str::<Settings>(&s).unwrap_or_default(),
            Err(_) => Settings::default(),
        },
        Err(_) => Settings::default(),
    };
    if let Ok(Some(v)) = db::get_setting(&db.0, CLOSE_TO_TRAY_KEY).await {
        settings.minimize_to_tray = v == "tray";
    }
    Ok(settings)
}

/// 写入偏好：theme/locale/autoStart 落 settings.json；`minimizeToTray` 桥接到 DB
/// `close_to_tray` 键并同步托盘菜单勾选态。
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    db: State<'_, AppDb>,
    settings: Settings,
) -> Result<(), String> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建设置目录失败: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| format!("序列化设置失败: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("写入设置失败: {e}"))?;

    let val = if settings.minimize_to_tray { "tray" } else { "exit" };
    db::set_setting(&db.0, CLOSE_TO_TRAY_KEY, val)
        .await
        .map_err(|e| db::db_err(e, "写入关闭行为失败"))?;

    let menu = app.state::<crate::TrayMenuState>();
    let _ = menu.close_to_tray.set_checked(settings.minimize_to_tray);
    Ok(())
}
