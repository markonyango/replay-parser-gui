use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use parser_lib::{replay::ReplayInfo, Chain, GameData};
use tauri::{AppHandle, Manager};
use tracing::debug;

use self::{
    error::{ParserAppError, ParserAppResult},
    game::{copy_replay_file, notify_main_window, ExtendedGameInformation},
    logfile::{LogfileGameInfo, LogfileGameList},
    status::{set_indicator, set_text, IndicatorState},
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

fn parse_replay_file(replay_file_path: &Path) -> ParserAppResult<ReplayInfo> {
    // parser_lib only takes a String; a lossy path would point at a different file.
    let replay_file_path = replay_file_path
        .to_str()
        .ok_or_else(|| ParserAppError::NonUtf8ReplayPath(replay_file_path.to_path_buf()))?;
    let parsed_replay = parser_lib::parse_raw(replay_file_path.to_owned())?;
    Ok(parsed_replay)
}

/// Checks that the logfile game belongs to the replay, so a replay is never uploaded
/// with the ID and results of another match.
///
/// Every player the logfile identifies must be in the replay. The replay may hold
/// more players: AI and dropped players can have no relic id in the logfile (0).
fn ensure_same_match(replay: &ReplayInfo, logfile_game: &LogfileGameInfo) -> ParserAppResult<()> {
    let replay_relic_ids = replay
        .players
        .iter()
        .filter_map(|chunk| match chunk {
            parser_lib::chunky::Chunk::Player(player) => Some(player.relic_id),
            _ => None,
        })
        .collect::<HashSet<_>>();

    let logfile_relic_ids = logfile_game
        .players
        .iter()
        .map(|player| player.relic_id as u64)
        .filter(|relic_id| *relic_id != 0)
        .collect::<HashSet<_>>();

    if logfile_relic_ids.is_empty() {
        return Err(ParserAppError::MatchMismatch(format!(
            "logfile game {} has no identified players",
            logfile_game.id
        )));
    }

    let mut missing = logfile_relic_ids
        .difference(&replay_relic_ids)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        missing.sort();
        return Err(ParserAppError::MatchMismatch(format!(
            "players {missing:?} of logfile game {} are not in the replay",
            logfile_game.id
        )));
    }

    Ok(())
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
        (
            "replay",
            "Replay folder",
            playback_found,
            playback_path.parent(),
        ),
        (
            "logfile",
            "Logfile",
            logfile_found,
            Some(logfile_path.as_path()),
        ),
    ] {
        let state = if found {
            IndicatorState::Ok
        } else {
            IndicatorState::Error
        };
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

    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx)?;

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
    debug!("Parsing logfile: {:?}", logfile_path);
    let logfile_game_info = parse_logfile(logfile_path)?;

    debug!("Parsing replay file: {:?}", replay_file_path);
    let mut replay_file_info = parse_replay_file(replay_file_path)?;

    Chain::with_defaults(steam_game_location)
        .run(&mut replay_file_info)
        .map_err(|e| ParserAppError::ParserLibError(format!("{e:?}")))?;

    ensure_same_match(&replay_file_info, &logfile_game_info)?;

    let mut replay_info = ExtendedGameInformation::from(&replay_file_info, &logfile_game_info);

    // Keep the local backup even if the upload fails; a failed copy must not hide the match.
    if let Err(e) = copy_replay_file(&replay_info, replay_file_path) {
        tracing::error!("{e}");
    }

    let replay_file_path = replay_file_path.to_path_buf();
    replay_info.send_replay_to_server(&replay_file_path)?;

    let (state, text) = if replay_info.status.is_accepted() {
        (IndicatorState::Ok, format!("Uploaded match {}", replay_info.id))
    } else {
        (
            IndicatorState::Error,
            format!("Upload of match {} failed", replay_info.id),
        )
    };
    set_indicator(
        handle,
        "upload",
        "Upload",
        state,
        replay_info.status.detail().map(str::to_owned),
    );
    set_text(handle, text);

    notify_main_window(&replay_info, main_window_handle)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use parser_lib::chunky::Chunk;

    use super::*;
    use crate::core::player_info::LogfilePlayerInfo;

    const FIXTURE_REPLAY: &str = "3v3.rec";

    fn replay_relic_ids(replay: &ReplayInfo) -> Vec<usize> {
        replay
            .players
            .iter()
            .filter_map(|chunk| match chunk {
                Chunk::Player(player) => Some(player.relic_id as usize),
                _ => None,
            })
            .collect()
    }

    fn logfile_game(relic_ids: impl IntoIterator<Item = usize>) -> LogfileGameInfo {
        LogfileGameInfo {
            players: relic_ids
                .into_iter()
                .map(|relic_id| LogfilePlayerInfo {
                    relic_id,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn accepts_logfile_game_with_the_replays_players() {
        let replay = parse_replay_file(Path::new(FIXTURE_REPLAY)).unwrap();
        let mut relic_ids = replay_relic_ids(&replay);
        relic_ids.reverse();
        // AI and unresolved dropped players carry relic id 0 and are ignored.
        relic_ids.push(0);

        assert!(ensure_same_match(&replay, &logfile_game(relic_ids)).is_ok());
    }

    #[test]
    fn accepts_replay_with_players_the_logfile_could_not_resolve() {
        let replay = parse_replay_file(Path::new(FIXTURE_REPLAY)).unwrap();
        let relic_ids = replay_relic_ids(&replay);

        let logfile = logfile_game(relic_ids.into_iter().skip(1));

        assert!(ensure_same_match(&replay, &logfile).is_ok());
    }

    #[test]
    fn rejects_logfile_game_with_a_player_not_in_the_replay() {
        let replay = parse_replay_file(Path::new(FIXTURE_REPLAY)).unwrap();
        let mut relic_ids = replay_relic_ids(&replay);
        relic_ids[0] = 1;

        let result = ensure_same_match(&replay, &logfile_game(relic_ids));

        assert!(matches!(result, Err(ParserAppError::MatchMismatch(_))));
    }

    #[test]
    fn rejects_logfile_game_without_known_players() {
        let replay = parse_replay_file(Path::new(FIXTURE_REPLAY)).unwrap();

        // E.g. the newest game block before its results were written.
        for logfile in [logfile_game([]), logfile_game([0, 0])] {
            let result = ensure_same_match(&replay, &logfile);
            assert!(matches!(result, Err(ParserAppError::MatchMismatch(_))));
        }
    }
}
