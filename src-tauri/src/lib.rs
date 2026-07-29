mod commands;
mod crypto;
mod git_sync;
mod http_service;
mod models;
mod storage;

use commands::AppData;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let storage = storage::Storage::new();
    let app_data = AppData {
        storage: Arc::new(Mutex::new(storage)),
    };

    let http_storage = app_data.storage.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_data)
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let storage_clone = http_storage.clone();
            tauri::async_runtime::spawn(async move {
                http_service::start_http_server(storage_clone).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_entries,
            commands::get_entry,
            commands::add_entry,
            commands::edit_entry,
            commands::delete_entry,
            commands::change_master_password,
            commands::init_password,
            commands::verify_password,
            commands::is_initialized,
            commands::get_config,
            commands::update_config,
            commands::git_push,
            commands::git_pull,
            commands::generate_totp,
            commands::generate_password,
            commands::export_json,
            commands::import_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
