use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parser_lib::{replay::ReplayInfo, Chain, GameData};
use tauri::{AppHandle, Manager};

use self::{
    error::{ParserAppError, ParserAppResult},
    game::{copy_replay_file, notify_main_window, ExtendedGameInformation},
    logfile::{LogfileGameInfo, LogfileGameList},
};

mod discover;
pub mod error;
pub mod game;
mod logfile;
mod paths;
pub mod player_info;
mod replay_reporter_dto;
mod transformer;

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
    let game_dir = paths::resolve_game_dir(
        &handle.path().document_dir()?,
        handle.path().home_dir().ok().as_deref(),
    );

    let logfile_path = game_dir.join("Logfiles").join("warnings.txt");
    let playback_path = game_dir.join("Playback").join("temp.rec");

    if !(logfile_path).exists() {
        return Err(ParserAppError::LogfileNotFoundError);
    }

    if !playback_path.parent().is_some_and(Path::is_dir) {
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

    // Find Steam installation location
    let loc = discover::locate_elite_module()
        .ok()
        .map(|m| GameData::open(&m))
        .transpose()
        .map_err(|_e| ParserAppError::GenericError("Steam location not found".to_string()))?;

    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx).unwrap();

    let watch_dir = replay_file_path.parent().ok_or_else(|| {
        ParserAppError::GenericError("Replay path has no parent directory".into())
    })?;

    debouncer
        .watcher()
        .watch(watch_dir, RecursiveMode::NonRecursive)?;

    for events in rx {
        // The directory also reports other files in Playback/; only react to temp.rec.
        // One debounced batch triggers a single upload, however many events it holds.
        if !events?.iter().any(|e| e.path == replay_file_path) {
            continue;
        }

        tracing::debug!("Received a replay file notify event");

        // A single unusable replay or logfile must not stop the watcher.
        if let Err(e) = process_replay(
            &replay_file_path,
            &logfile_path,
            loc.as_ref(),
            &main_window_handle,
        ) {
            tracing::error!("Skipping replay, could not process it: {}", e);
        }
    }

    Ok(())
}

fn process_replay(
    replay_file_path: &Path,
    logfile_path: &Path,
    steam_game_location: Option<&GameData>,
    main_window_handle: &tauri::WebviewWindow,
) -> ParserAppResult<()> {
    let logfile_game_info = parse_logfile(logfile_path)?;
    let mut replay_file_info = parse_replay_file(replay_file_path.to_str().unwrap().to_string())?;

    Chain::with_defaults(steam_game_location)
        .run(&mut replay_file_info)
        .map_err(|e| ParserAppError::ParserLibError(format!("{e:?}")))?;

    let mut replay_info = ExtendedGameInformation::from(&replay_file_info, &logfile_game_info);

    let replay_file_path = replay_file_path.to_path_buf();
    replay_info.send_replay_to_server(&replay_file_path)?;

    copy_replay_file(&replay_info, &replay_file_path)?;
    notify_main_window(&replay_info, main_window_handle)?;

    Ok(())
}
