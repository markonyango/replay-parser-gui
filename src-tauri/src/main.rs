#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;

use crate::core::{
    error::{ParserAppError, ParserAppResult},
    game::{ExtendedGameInformation, LogfileGameList},
};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use parser_lib::{self};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use tauri::api::path::document_dir;
use tauri::{Manager};

#[tauri::command]
fn parse_file(path: &str) -> Option<String> {
    let file_path = path.to_string();
    parser_lib::parse_file(file_path)
}

fn update_game_list() -> ParserAppResult<String> {
    let InputFiles {
        replay_file_path,
        logfile_path,
    } = get_input_files()?;

    let mut game_list = LogfileGameList::new();
    game_list.read_logfile(&logfile_path)?;
    game_list.parse()?;

    let parsed_replay = parser_lib::parse_raw(replay_file_path.to_str().unwrap().to_string())
        .map_err(|error| ParserAppError::ParserLibError(error.to_string()))?;

    let replay_info = ExtendedGameInformation::from(parsed_replay, game_list.games.last().unwrap());

    // Copy replay file to ESL folder
    copy_replay_file(replay_file_path, &replay_info)?;

    serde_json::to_string_pretty(&replay_info)
        .map_err(|error| ParserAppError::ParserLibError(error.to_string()))
}

fn main() {
    color_eyre::install().unwrap();

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle();

            std::thread::spawn(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
                let InputFiles { replay_file_path, logfile_path: _ } = get_input_files().unwrap();

                watcher
                    .watch(Path::new(replay_file_path.as_os_str()), RecursiveMode::Recursive)
                    .unwrap();

                for res in rx {
                    match res {
                        Ok(event) => if let notify::EventKind::Modify(_) = event.kind {
                            println!("changed; {:?}", event);

                            if let Ok(replay_info) = update_game_list() {
                                handle
                                    .get_window("main")
                                    .unwrap()
                                    .emit_all("new-game", replay_info)
                                    .unwrap();
                            }
                        },
                        Err(e) => println!("watch error: {:?}", e),
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct InputFiles {
    replay_file_path: PathBuf,
    logfile_path: PathBuf,
}
fn get_input_files() -> ParserAppResult<InputFiles> {
    let Some(documentdir_path) = document_dir() else {
        return Err(ParserAppError::ParserLibError(
            "Could not find documents directory!".into(),
        ));
    };

    let mut game_path = documentdir_path;
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

fn copy_replay_file(
    replay_file_path: PathBuf,
    replay_info: &ExtendedGameInformation,
) -> ParserAppResult<()> {
    let mut file_name = replay_file_path.clone();
    let map_name = replay_info.map.path.replace("DATA:maps\\pvp\\", "");

    file_name.set_file_name(format!("{}_{}.rec", replay_info.id, map_name));

    let Ok(_) = fs::copy(replay_file_path, file_name) else {
        return Err(ParserAppError::ParserLibError("Could not copy replay file".into()));
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[cfg(unix)]
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

    #[test]
    fn can_copy_replay_file() {
        let replay_info = ExtendedGameInformation {
            id: 1234,
            map: parser_lib::chunky::Map {
                path: "DATA:maps\\pvp\\6p_estia".into(),
                ..Default::default()
            },
            ..Default::default()
        };

        let replay_file_path = Path::new("3v3.rec");

        let result = copy_replay_file(replay_file_path.into(), &replay_info);

        let removed = fs::remove_file("1234_6p_estia.rec");

        assert!(result.is_ok());
        assert!(removed.is_ok());
    }
}
