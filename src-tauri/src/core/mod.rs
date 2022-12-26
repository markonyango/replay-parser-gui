use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parser_lib::replay::ReplayInfo;
use serde_json::json;
use tauri::{api::path::document_dir, AppHandle, Manager};
use tracing::{error, info};

use self::{
    error::{ParserAppError, ParserAppResult},
    game::ExtendedGameInformation,
    logfile::{LogfileGameInfo, LogfileGameList},
    replay_reporter_dto::ReplayReportDto,
};

pub mod error;
pub mod game;
mod logfile;
pub mod player_info;
mod replay_reporter_dto;

pub struct InputFiles {
    replay_file_path: PathBuf,
    logfile_path: PathBuf,
}

pub fn parse_logfile(logfile_path: &Path) -> error::ParserAppResult<LogfileGameInfo> {
    let mut game_list = LogfileGameList::new();
    game_list.read_logfile(logfile_path)?;
    game_list.parse()?;

    let Some(last_game) = game_list.games.last() else {
        return Err(ParserAppError::ParserLibError("Could not get last game from list of games in logfile".into()));
    };

    Ok(last_game.to_owned())
}

fn parse_replay_file(replay_file_path: String) -> ParserAppResult<ReplayInfo> {
    let parsed_replay = parser_lib::parse_raw(replay_file_path)?;
    Ok(parsed_replay)
}

lazy_static! {
    static ref PLAYBACK_PATH: PathBuf = document_dir()
        .unwrap()
        .join(r"My Games\Dawn of War II - Retribution\Playback\temp.rec");
    static ref LOGFILE_PATH: PathBuf = document_dir()
        .unwrap()
        .join(r"My Games\Dawn of War II - Retribution\Logfiles\warnings.txt");
}

pub fn get_input_files() -> ParserAppResult<InputFiles> {
    if !(*LOGFILE_PATH).exists() {
        return Err(ParserAppError::LogfileNotFoundError);
    }

    if !(*PLAYBACK_PATH).exists() {
        return Err(ParserAppError::ReplayNotFoundError);
    }

    Ok(InputFiles {
        replay_file_path: (*PLAYBACK_PATH).to_path_buf(),
        logfile_path: (*LOGFILE_PATH).to_path_buf(),
    })
}

pub fn handle_new_game_event(handle: AppHandle) -> ParserAppResult<()> {
    let Some(main_window_handle) = handle.get_window("main") else {
                    return Err(ParserAppError::GenericError("Could not acquire main window handle. This is unrecoverable".into()));
                };

    let (tx, rx) = std::sync::mpsc::channel();
    // let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let InputFiles {
        replay_file_path,
        logfile_path,
    } = get_input_files()?;

    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx).unwrap();

    debouncer.watcher().watch(
        Path::new(replay_file_path.as_os_str()),
        RecursiveMode::Recursive,
    )?;

    for events in rx {
        for _e in events? {
            tracing::debug!("Received a replay file notify event");
            let logfile_game_info = parse_logfile(&logfile_path)?;
            let replay_file_info =
                parse_replay_file(replay_file_path.to_str().unwrap().to_string())?;
            
            let mut replay_info = ExtendedGameInformation::new();
            replay_info
                .from(replay_file_info, &logfile_game_info)
                .copy_replay_file(&replay_file_path)?
                .transform_replay_to_base64(&replay_file_path)?
                .send_replay_to_server()?
                .notify_main_window(&main_window_handle)?;

            // Copy replay file to ESL folder
            // copy_replay_file(&replay_file_path, &replay_info)?;

            // replay_info.replay = transform_replay_to_base64(&replay_file_path).ok();

            // send_replay_to_server(&mut replay_info)?;

            // replay_info.replay = None;

            // let json = serde_json::to_string_pretty(&replay_info)?;
            // main_window_handle.emit_all("new-game", json)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use parser_lib::chunky::Map;

    use super::*;

    #[cfg(unix)]
    #[test]
    fn merged_replay_json_is_correct() {
        // Mock Documents folder to be our test folder so we can control
        // game files content
        std::env::set_var("HOME", "./test");

        let replay_info = parse_logfile();
        assert!(replay_info.is_ok());

        let replay_info = replay_info.unwrap();

        let games = serde_json::to_string_pretty(&replay_info).unwrap();

        assert!(games.contains("Raubritter"));
        assert!(games.contains("JamezNunes"));
        assert!(games.contains("Cerano"));
        assert!(games.contains("Venniie"));
        assert!(games.contains("[SB]Odium"));
        assert!(games.contains("Morgan MLGman"));
    }

    #[test]
    fn yields_correct_game_paths() {
        let InputFiles {
            replay_file_path,
            logfile_path,
        } = get_input_files().unwrap();
        assert!(replay_file_path.exists());
        assert!(logfile_path.exists());
    }

    #[test]
    fn can_send_json_to_server() {
        let mut replay_info = ExtendedGameInformation {
            dev: Some(true),
            replay: Some("ABC".into()),
            status: "ok".into(),
            id: 1234,
            name: "".into(),
            mod_chksum: 1234,
            mod_version: 1234,
            md5: "".into(),
            date: "".into(),
            ticks: 123,
            game: game::GameInfo {
                name: "".into(),
                mode: "".into(),
                resources: "".into(),
                locations: "".into(),
                victory_points: 500,
            },
            map: Map {
                name: "todo!()".to_string(),
                description: "todo!()".to_string(),
                abbrname: "todo!()".to_string(),
                maxplayers: 6,
                path: "todo!()".to_string(),
                date: "todo!()".to_string(),
                width: 512,
                height: 512,
            },
            players: vec![],
            messages: vec![],
            actions: vec![],
            aborted: false,
            frames: 123,
            ended_at: "".into(),
        };

        let res = send_replay_to_server(&mut replay_info);
        assert!(res.is_ok());
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

        let result = copy_replay_file(&replay_file_path.into(), &replay_info);

        let removed = fs::remove_file("1234_6p_estia.rec");

        assert!(result.is_ok());
        assert!(removed.is_ok());
    }
}
