#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;

use crate::core::{
    error::{ParserAppError, ParserAppResult},
    game::{ExtendedGameInformation, LogfileGameList},
};
use parser_lib::{self};
use std::path::PathBuf;
use tauri::api::path::document_dir;
use tauri_plugin_fs_watch::Watcher;

#[tauri::command]
fn parse_file(path: &str) -> Option<String> {
    let file_path = path.to_string();
    parser_lib::parse_file(file_path)
}

#[tauri::command]
fn update_game_list() -> ParserAppResult<String> {
    if let Some(documentdir_path) = document_dir() {
        let InputFiles {
            replay_file_path,
            logfile_path,
        } = get_input_files(documentdir_path)?;

        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(&logfile_path)?;
        game_list.parse()?;

        let parsed_replay = parser_lib::parse_raw(replay_file_path.to_str().unwrap().to_string())
            .map_err(|error| ParserAppError::ParserLibError(error.to_string()))?;

        let result = ExtendedGameInformation::from(parsed_replay, game_list.games.last().unwrap());

        serde_json::to_string_pretty(&result)
            .map_err(|error| ParserAppError::ParserLibError(error.to_string()))
    } else {
        return Err(ParserAppError::ParserLibError(
            "Could not find documents directory!".into(),
        ));
    }
}

fn main() {
    color_eyre::install().unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_file, update_game_list])
        .plugin(Watcher::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct InputFiles {
    replay_file_path: PathBuf,
    logfile_path: PathBuf,
}
fn get_input_files(document_dir_path: PathBuf) -> ParserAppResult<InputFiles> {
    let mut game_path = document_dir_path;
    game_path.push("My Games");
    game_path.push("Dawn of War II - Retribution");

    let mut logfile_path = game_path.clone();
    logfile_path.push("Logfiles");
    logfile_path.push("warnings.txt");

    let mut replay_file_path = game_path.clone();
    replay_file_path.push("Playback");
    replay_file_path.push("temp.rec");

    if !logfile_path.exists() {
        return Err(ParserAppError::LogfileNotFoundError);
    }

    if !replay_file_path.exists() {
        return Err(ParserAppError::ReplayNotFoundError);
    }

    Ok(InputFiles {
        replay_file_path,
        logfile_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_replay_json_is_correct() {
        // Mock Documents folder to be our test folder so we can control
        // game files content
        std::env::set_var("HOME", "./test");

        let game_list = update_game_list();
        assert!(game_list.is_ok());

        let games = game_list.unwrap();

        assert!(games.contains("Raubritter"));
        assert!(games.contains("JamezNunes"));
        assert!(games.contains("Cerano"));
        assert!(games.contains("Venniie"));
        assert!(games.contains("[SB]Odium"));
        assert!(games.contains("Morgan MLGman"));
    }
}
