use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parser_lib::replay::ReplayInfo;
use tauri::{api::path::document_dir, AppHandle, Manager};

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
        }
    }

    Ok(())
}

