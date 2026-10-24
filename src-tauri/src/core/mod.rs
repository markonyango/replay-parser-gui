use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parser_lib::{replay::ReplayInfo, Chain, GameData};
use tauri::{AppHandle, Manager};

use self::{
    status::{set_indicator, set_text, IndicatorState},
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
pub mod status;
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

    let logfile_found = logfile_path.exists();
    let playback_found = playback_path.parent().is_some_and(Path::is_dir);

    for (id, label, found, path) in [
        ("replay", "Replay folder", playback_found, playback_path.parent()),
        ("logfile", "Logfile", logfile_found, Some(logfile_path.as_path())),
    ] {
        let state = if found { IndicatorState::Ok } else { IndicatorState::Error };
        let tooltip = path.map(|p| p.display().to_string());
        set_indicator(handle, id, label, state, tooltip);
    }

    if !logfile_found {
        return Err(ParserAppError::LogfileNotFoundError);
    }

    if !playback_found {
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

    set_text(handle, "Watching for replays");

    for events in rx {
        // The directory also reports other files in Playback/; only react to temp.rec.
        // One debounced batch triggers a single upload, however many events it holds.
        if !events?.iter().any(|e| e.path == replay_file_path) {
            continue;
        }

        tracing::debug!("Received a replay file notify event");
        set_text(handle, "Processing replay…");

        // A single unusable replay or logfile must not stop the watcher.
        if let Err(e) = process_replay(
            &replay_file_path,
            &logfile_path,
            loc.as_ref(),
            &main_window_handle,
            handle,
        ) {
            tracing::error!("Skipping replay, could not process it: {}", e);
            set_text(handle, format!("Could not process replay: {e}"));
        }
    }

    Ok(())
}

fn process_replay(
    replay_file_path: &Path,
    logfile_path: &Path,
    steam_game_location: Option<&GameData>,
    main_window_handle: &tauri::WebviewWindow,
    handle: &AppHandle,
) -> ParserAppResult<()> {
    let logfile_game_info = parse_logfile(logfile_path)?;
    let mut replay_file_info = parse_replay_file(replay_file_path.to_str().unwrap().to_string())?;

    Chain::with_defaults(steam_game_location)
        .run(&mut replay_file_info)
        .map_err(|e| ParserAppError::ParserLibError(format!("{e:?}")))?;

    let mut replay_info = ExtendedGameInformation::from(&replay_file_info, &logfile_game_info);

    let replay_file_path = replay_file_path.to_path_buf();
    replay_info.send_replay_to_server(&replay_file_path)?;

    // Mirrors the heuristic used when the server responds: any "error" in the status is a failure.
    let upload_failed = replay_info.status.contains("error");
    set_indicator(
        handle,
        "upload",
        "Upload",
        if upload_failed { IndicatorState::Error } else { IndicatorState::Ok },
        Some(replay_info.status.clone()),
    );
    set_text(
        handle,
        if upload_failed {
            format!("Upload of match {} failed", replay_info.id)
        } else {
            format!("Uploaded match {}", replay_info.id)
        },
    );

    copy_replay_file(&replay_info, &replay_file_path)?;
    notify_main_window(&replay_info, main_window_handle)?;

    Ok(())
}
