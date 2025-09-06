use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parser_lib::replay::ReplayInfo;
use tauri::{path::PathResolver, AppHandle, Manager};

use self::{
    error::{ParserAppError, ParserAppResult},
    game::ExtendedGameInformation,
    logfile::{LogfileGameInfo, LogfileGameList},
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
        return Err(ParserAppError::ParserLibError(
            "Could not get last game from list of games in logfile".into(),
        ));
    };

    Ok(last_game.to_owned())
}

fn parse_replay_file(replay_file_path: String) -> ParserAppResult<ReplayInfo> {
    let parsed_replay = parser_lib::parse_raw(replay_file_path)?;
    Ok(parsed_replay)
}

pub fn get_input_files(handle: &AppHandle) -> ParserAppResult<InputFiles> {
    let logfile_path = handle
        .path()
        .document_dir()?
        .join("My Games")
        .join("Dawn of War II - Retribution")
        .join("Logfiles")
        .join("warnings.txt");

    let playback_path = handle
        .path()
        .document_dir()?
        .join("My Games")
        .join("Dawn of War II - Retribution")
        .join("Playback")
        .join("temp.rec");

    tracing::debug!("Replay path: {playback_path:?}");
    tracing::debug!("Log path: {logfile_path:?}");

    if !(logfile_path).exists() {
        return Err(ParserAppError::LogfileNotFoundError);
    }

    if !(playback_path).exists() {
        return Err(ParserAppError::ReplayNotFoundError);
    }

    Ok(InputFiles {
        replay_file_path: (playback_path).to_path_buf(),
        logfile_path: (logfile_path).to_path_buf(),
    })
}

pub fn handle_new_game_event(handle: &AppHandle) -> ParserAppResult<()> {
    let Some(main_window_handle) = handle.get_webview_window("main") else {
        return Err(ParserAppError::GenericError(
            "Could not acquire main window handle. This is unrecoverable".into(),
        ));
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let InputFiles {
        replay_file_path,
        logfile_path,
    } = get_input_files(&handle)?;


    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx).unwrap();

    debouncer.watcher().watch(
        Path::new(replay_file_path.as_os_str()),
        RecursiveMode::Recursive,
    )?;

    for events in rx {
        for _e in events? {
            tracing::info!("Received a replay file notify event");
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
        }
    }

    Ok(())
}
