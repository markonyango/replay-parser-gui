#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate lazy_static;

mod core;

use crate::core::handle_new_game_event;
use tracing_subscriber;

fn main() {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt().init();

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle();

            std::thread::spawn(move || handle_new_game_event(handle));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
