// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod db;
mod models;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let pool = tauri::async_runtime::block_on(db::init_db(app.handle()))?;
            app.manage(db::AppDb(pool));
            app.manage(db::AppConnState(std::sync::RwLock::new(String::from("idle"))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::hosts::list_hosts,
            commands::hosts::get_host,
            commands::hosts::save_host,
            commands::hosts::delete_host,
            commands::rules::list_rules,
            commands::rules::add_rule,
            commands::rules::update_rule,
            commands::rules::delete_rule,
            commands::logs::add_log,
            commands::logs::list_logs,
            commands::logs::clear_logs,
            commands::logs::read_logs,
            commands::tunnel::get_conn_state,
            commands::tunnel::set_conn_state,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::pick_key_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}