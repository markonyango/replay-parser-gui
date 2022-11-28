#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;

use std::path::PathBuf;

use crate::core::game::ExtendedGameInformation;
use crate::core::game::LogfileGameList;

use color_eyre::{eyre::eyre, Result};
use parser_lib::{self};
use tauri::api::Error;
use tauri::api::path::document_dir;
use tauri_plugin_fs_watch::Watcher;

#[tauri::command]
fn parse_file(path: &str) -> Option<String> {
    let file_path = path.to_string();
    parser_lib::parse_file(file_path)
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum ParserAppError {
    #[error("Parser lib error: {0}")]
    ParserLibError(String),
    #[error("Get Input files error: {0}")]
    InputFilesError(String)
}

#[tauri::command]
fn update_game_list() -> tauri::Result<String> {
    let documentdir_path = document_dir().ok_or_else(|| tauri::Error::FailedToExecuteApi(Error::Path("Could not find documents directory".into())))?;

    let InputFiles { replay_file_path, logfile_path } = get_input_files(documentdir_path).map_err(|error| ParserAppError::InputFilesError(error.to_string()))?;
    
    let mut game_list = LogfileGameList::new();
    game_list.read_logfile(&logfile_path).unwrap();
    game_list.parse();

    let parsed_replay = parser_lib::parse_raw(replay_file_path.to_str().unwrap().to_string()).map_err(|error| ParserAppError::ParserLibError(error.to_string()))?;
    
    let result = ExtendedGameInformation::from(parsed_replay, game_list.games.last().unwrap());

    serde_json::to_string_pretty(&result)
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
fn get_input_files(document_dir_path: PathBuf) -> Result<InputFiles> {
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
        return Err(eyre!("Could not find logfile."));
    }

    if !replay_file_path.exists() {
        return Err(eyre!("Could not find replay file."));
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
        let mut logfile_path = PathBuf::new();
        logfile_path.push("warnings_confirmed.txt");

        assert!(logfile_path.exists());
    }
}
