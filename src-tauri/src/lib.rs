// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod crypto;
mod db;
mod models;
mod settings;
mod ssh;

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

use commands::settings::CLOSE_TO_TRAY_KEY;

/// 托盘菜单项集合：需要在事件回调中更新文案与勾选态，故常驻 managed state
struct TrayMenuState {
    show: MenuItem<tauri::Wry>,
    close_to_tray: CheckMenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let pool = tauri::async_runtime::block_on(db::init_db(app.handle()))?;
            // 初始化敏感字段加密密钥，并把历史明文密码/私钥路径迁移为密文
            crypto::init_master_key(&app.path().app_data_dir()?)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            tauri::async_runtime::block_on(migrate_plaintext_hosts(&pool))?;

            // 读取已保存的关闭行为偏好，用于托盘菜单初始勾选态
            let close_pref = tauri::async_runtime::block_on(db::get_setting(&pool, CLOSE_TO_TRAY_KEY))
                .ok()
                .flatten();

            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let close_to_tray = CheckMenuItem::with_id(
                app,
                "toggle_close_to_tray",
                "关闭时最小化到托盘",
                true,
                close_pref.as_deref() == Some("tray"),
                None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &close_to_tray, &quit])?;
            app.manage(TrayMenuState {
                show,
                close_to_tray,
                quit,
            });

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().expect("缺少默认应用图标").clone())
                .tooltip("sshportforward")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    // 左键单击托盘图标 → 显示并聚焦主窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "toggle_close_to_tray" => {
                        // 菜单点击后勾选态已自动翻转，按翻转后的值写库
                        let checked = app
                            .state::<TrayMenuState>()
                            .close_to_tray
                            .is_checked()
                            .unwrap_or(false);
                        let value = if checked { "tray" } else { "exit" };
                        let db = app.state::<db::AppDb>();
                        if let Err(e) =
                            tauri::async_runtime::block_on(db::set_setting(&db.0, CLOSE_TO_TRAY_KEY, value))
                        {
                            eprintln!("保存关闭行为设置失败: {e}");
                        }
                        // 通知前端实时同步设置面板的复选框状态
                        let _ = app.emit("close-to-tray-changed", checked);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            app.manage(db::AppDb(pool));
            app.manage(db::AppConnState(std::sync::RwLock::new(String::from("idle"))));
            app.manage(ssh::TunnelState::default());
            Ok(())
        })
        .on_window_event(|window, event| {
            // 拦截关闭：仅在连接中且未确定最小化到托盘时弹窗询问；
            // 已选择托盘则直接隐藏；未连接则按偏好退出或隐藏，不弹窗
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let db = app.state::<db::AppDb>();
                let conn = app.state::<db::AppConnState>();
                let conn_state = conn
                    .0
                    .read()
                    .expect("连接状态锁已中毒")
                    .clone();
                let pref = tauri::async_runtime::block_on(db::get_setting(&db.0, CLOSE_TO_TRAY_KEY))
                    .ok()
                    .flatten();
                let connected = conn_state == "connected";
                match (connected, pref.as_deref()) {
                    // 已选择最小化到托盘：无论是否连接，直接隐藏，不弹窗
                    (_, Some("tray")) => {
                        api.prevent_close();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    // 连接中且未确定最小化到托盘（未设置或为 exit）：弹窗询问
                    (true, _) => {
                        api.prevent_close();
                        let _ = app.emit("ask-close-behavior", ());
                    }
                    // 未连接且未选择托盘：直接退出，不弹窗
                    (false, _) => {
                        // 放行关闭，应用退出
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::hosts::list_hosts,
            commands::hosts::save_host,
            commands::hosts::delete_host,
            commands::rules::add_rule,
            commands::rules::update_rule,
            commands::rules::delete_rule,
            commands::logs::add_log,
            commands::logs::clear_logs,
            commands::tunnel::get_conn_state,
            commands::tunnel::get_traffic,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::pick_key_file,
            commands::tunnel::inspect_private_key,
            commands::settings::get_app_setting,
            commands::settings::set_app_setting,
            commands::settings::hide_main_window,
            commands::settings::exit_app,
            commands::settings::set_tray_texts,
            settings::get_settings,
            settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 显示并聚焦主窗口（托盘左键点击 / 菜单"显示主窗口"）
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// 一次性迁移：把 hosts 表中历史明文的 password / key_path 加密写回。
/// 已是 `enc1:` 密文的行跳过，避免二次加密。
async fn migrate_plaintext_hosts(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id, password, key_path FROM hosts")
            .fetch_all(pool)
            .await?;
    for (id, password, key_path) in rows {
        let (enc_password, password_changed) = if crypto::is_encrypted(&password) {
            (password.clone(), false)
        } else {
            (crypto::encrypt(&password)?, true)
        };
        let (enc_key_path, key_changed) = if crypto::is_encrypted(&key_path) {
            (key_path.clone(), false)
        } else {
            (crypto::encrypt(&key_path)?, true)
        };
        if !password_changed && !key_changed {
            continue; // 两个字段均已是密文，无需更新
        }
        sqlx::query("UPDATE hosts SET password = ?, key_path = ? WHERE id = ?")
            .bind(&enc_password)
            .bind(&enc_key_path)
            .bind(&id)
            .execute(pool)
            .await?;
    }
    Ok(())
}
