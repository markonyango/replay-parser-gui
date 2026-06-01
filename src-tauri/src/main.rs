#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate lazy_static;

mod core;
mod logger;

use crate::{core::handle_new_game_event, logger::init_logging};
use tauri::Manager;

fn should_start_minimized() -> bool {
    std::env::args().any(|arg| arg == "--start-minimized")
}

fn main() {
    color_eyre::install().unwrap();

    let start_minimized = should_start_minimized();

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
        .setup(move |app| {
            init_logging(&app);
            let handle = app.handle().clone();

            if let Some(window) = app.get_webview_window("main") {
                if start_minimized {
                    if let Err(e) = window.minimize() {
                        tracing::error!("Failed to minimize main window on startup: {e}");
                    }
                    if let Err(e) = window.show() {
                        tracing::error!("Failed to show minimized main window on startup: {e}");
                    }
                } else {
                    if let Err(e) = window.show() {
                        tracing::error!("Failed to show main window on startup: {e}");
                    }
                    if let Err(e) = window.set_focus() {
                        tracing::error!("Failed to focus main window on startup: {e}");
                    }
                }
            }

            std::thread::spawn(move || match handle_new_game_event(&handle) {
                Ok(_) => (),
                Err(e) => tracing::error!("Error occurred in app backend: {e}"),
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
