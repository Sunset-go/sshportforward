//! 应用设置读写、托盘菜单文案同步、关闭/启动提醒的原生对话框处理。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::db::{AppConnState, AppDb};

/// 设置键：关闭窗口时的行为（`tray` = 最小化到托盘，`exit` = 直接退出）。
pub const CLOSE_TO_TRAY_KEY: &str = "close_to_tray";

/// 前端界面语言状态（启动时由前端同步，`zh` 或 `en`），用于原生对话框选择文案
pub struct UiLocale(pub std::sync::Mutex<String>);

/// 防止连点关闭或快速重连时弹出多个对话框
static ASK_ACTIVE: AtomicBool = AtomicBool::new(false);

fn try_begin_ask() -> bool {
    ASK_ACTIVE.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
}

fn end_ask() {
    ASK_ACTIVE.store(false, Ordering::SeqCst);
}

fn current_locale(app: &AppHandle) -> String {
    app.state::<UiLocale>().0.lock().expect("语言锁已中毒").clone()
}

fn sync_tray_check(app: &AppHandle, value: &str) {
    if let Some(menu) = app.try_state::<crate::TrayMenuState>() {
        let _ = menu.close_to_tray.set_checked(value == "tray");
    }
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// 读取应用设置，键不存在返回 None
#[tauri::command]
pub async fn get_app_setting(db: State<'_, AppDb>, key: String) -> Result<Option<String>, String> {
    crate::db::get_setting(&db.0, &key)
        .await
        .map_err(|e| crate::db::db_err(e, "读取设置失败"))
}

/// 写入应用设置（存在则覆盖）；写入关闭行为时同步托盘菜单勾选态
#[tauri::command]
pub async fn set_app_setting(app: AppHandle, db: State<'_, AppDb>, key: String, value: String) -> Result<(), String> {
    crate::db::set_setting(&db.0, &key, &value)
        .await
        .map_err(|e| crate::db::db_err(e, "写入设置失败"))?;
    if key == CLOSE_TO_TRAY_KEY {
        sync_tray_check(&app, &value);
    }
    Ok(())
}

/// 前端同步当前界面语言（`zh` / `en`），供原生对话框选择文案
#[tauri::command]
pub fn set_ui_locale(app: AppHandle, locale: String) {
    *app.state::<UiLocale>().0.lock().expect("语言锁已中毒") = locale;
}

/// 按前端当前语言更新托盘菜单文案（启动与语言切换时调用）
#[tauri::command]
pub fn set_tray_texts(
    app: AppHandle,
    show: String,
    close_to_tray: String,
    quit: String,
) -> Result<(), String> {
    let menu = app.state::<crate::TrayMenuState>().inner();
    menu.show.set_text(show).map_err(|e| format!("更新托盘菜单失败: {e}"))?;
    menu.close_to_tray.set_text(close_to_tray).map_err(|e| format!("更新托盘菜单失败: {e}"))?;
    menu.quit.set_text(quit).map_err(|e| format!("更新托盘菜单失败: {e}"))?;
    Ok(())
}

// ─── 原生对话框业务逻辑（由 lib.rs / tunnel.rs 在异步任务中调用）────────────

/// 处理窗口关闭请求：
/// - 已有偏好（tray/exit）直接执行；
/// - 无偏好则弹系统原生对话框询问一次，写入数据库记住选择。
/// 使用原生对话框而非前端弹窗，前端是否加载完成不影响弹窗的显示。
pub async fn handle_close_requested(app: AppHandle) {
    if !try_begin_ask() {
        return;
    }
    let pool = app.state::<AppDb>().0.clone();
    let pref = crate::db::get_setting(&pool, CLOSE_TO_TRAY_KEY).await.ok().flatten();

    // 检查当前连接状态；若已连接且无偏好记录，先询问用户是否开启托盘最小化
    let conn_state = app
        .state::<AppConnState>()
        .0
        .read()
        .expect("连接状态锁已中毒")
        .clone();
    let is_connected = matches!(conn_state.as_str(), "connected" | "connecting" | "disconnecting");

    if is_connected && pref.is_none() {
        let is_zh = current_locale(&app) != "en";
        let (title, msg, ok, cancel) = if is_zh {
            (
                "断开连接并退出？",
                "当前仍处于连接状态。点击确定后将断开连接并退出；若希望保留连接，可先开启最小化到托盘。",
                "断开并退出",
                "开启托盘最小化",
            )
        } else {
            (
                "Disconnect and Exit?",
                "The tunnel is still connected. Click OK to disconnect and exit; or enable minimize-to-tray to keep the tunnel running.",
                "Disconnect & Exit",
                "Enable Tray Minimize",
            )
        };
        let choice = app
            .dialog()
            .message(msg)
            .title(title)
            .buttons(MessageDialogButtons::OkCancelCustom(ok.into(), cancel.into()))
            .kind(MessageDialogKind::Info)
            .blocking_show();
        end_ask();
        if choice {
            // 用户选「开启托盘最小化」→ 写入偏好并隐藏窗口（保留连接）
            if let Err(e) = crate::db::set_setting(&pool, CLOSE_TO_TRAY_KEY, "tray").await {
                eprintln!("保存关闭行为设置失败: {e}");
            }
            sync_tray_check(&app, "tray");
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }
        } else {
            // 用户选「断开并退出」→ 写入偏好并退出应用
            if let Err(e) = crate::db::set_setting(&pool, CLOSE_TO_TRAY_KEY, "exit").await {
                eprintln!("保存关闭行为设置失败: {e}");
            }
            sync_tray_check(&app, "exit");
            app.exit(0);
        }
        return;
    }

    // 未连接或已有偏好：按记录直接执行
    match pref.as_deref() {
        Some("tray") => {
            end_ask();
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }
        }
        _ => {
            end_ask();
            app.exit(0);
        }
    }
}

/// 连接成功后提醒开启最小化到托盘（由 tunnel.rs 调用）；
/// 已有偏好则静默跳过，避免隧道运行中误关窗口导致断连。
pub async fn ask_enable_tray_after_connect(app: AppHandle) {
    if !try_begin_ask() {
        return;
    }
    let pool = app.state::<AppDb>().0.clone();
    let pref = crate::db::get_setting(&pool, CLOSE_TO_TRAY_KEY).await.ok().flatten();
    if pref.as_deref() == Some("tray") {
        end_ask();
        return;
    }
    let is_zh = current_locale(&app) != "en";
    let (title, msg, ok, cancel) = if is_zh {
        (
            "开启最小化到托盘？",
            "当前未开启「关闭时最小化到托盘」。开启后，点击关闭窗口时程序将收容到托盘，隧道保持运行。是否开启？",
            "开启",
            "取消",
        )
    } else {
        (
            "Enable Minimize to Tray?",
            "\"Minimize to Tray on Close\" is not enabled. Once enabled, closing the window keeps the app in the tray and the tunnel running. Enable it now?",
            "Enable",
            "Cancel",
        )
    };
    let enable = app
        .dialog()
        .message(msg)
        .title(title)
        .buttons(MessageDialogButtons::OkCancelCustom(ok.into(), cancel.into()))
        .kind(MessageDialogKind::Info)
        .blocking_show();
    if enable {
        if let Err(e) = crate::db::set_setting(&pool, CLOSE_TO_TRAY_KEY, "tray").await {
            eprintln!("保存关闭行为设置失败: {e}");
        }
        sync_tray_check(&app, "tray");
    }
    end_ask();
}