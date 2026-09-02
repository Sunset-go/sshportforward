// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod crypto;
mod db;
mod models;
mod ssh;

use tauri::Manager;

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
            app.manage(db::AppDb(pool));
            app.manage(db::AppConnState(std::sync::RwLock::new(String::from("idle"))));
            app.manage(ssh::TunnelState::default());
            Ok(())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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