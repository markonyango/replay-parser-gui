#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate lazy_static;

mod core;
mod logger;

use tauri::Manager;

use crate::{
    core::{handle_new_game_event, status},
    logger::init_logging,
};

fn main() {
    color_eyre::install().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![status::get_status])
        .setup(|app| {
            init_logging(&app);
            app.manage(status::StatusStore::default());
            let handle = app.handle().clone();

            std::thread::spawn(move || match handle_new_game_event(&handle) {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Error occurred in app backend: {e}");
                    status::set_text(&handle, format!("Watcher stopped: {e}"));
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
