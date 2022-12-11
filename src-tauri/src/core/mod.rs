use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use tauri::{api::path::document_dir, AppHandle, Manager};

use self::{
    error::{ParserAppError, ParserAppResult},
    game::{ExtendedGameInformation, LogfileGameList},
    replay_reporter_dto::ReplayReportDto,
};

pub mod error;
pub mod game;
pub mod player_info;
mod replay_reporter_dto;

pub struct InputFiles {
    replay_file_path: PathBuf,
    logfile_path: PathBuf,
}

pub fn update_game_list() -> error::ParserAppResult<ExtendedGameInformation> {
    let InputFiles {
        replay_file_path,
        logfile_path,
    } = get_input_files()?;

    let mut game_list = LogfileGameList::new();
    game_list.read_logfile(&logfile_path)?;
    game_list.parse()?;

    let parsed_replay = parser_lib::parse_raw(replay_file_path.to_str().unwrap().to_string())
        .map_err(|error| ParserAppError::ParserLibError(error.to_string()))?;

    Ok(ExtendedGameInformation::from(
        parsed_replay,
        game_list.games.last().unwrap(),
    ))
}

pub fn get_input_files() -> ParserAppResult<InputFiles> {
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

pub fn copy_replay_file(
    replay_file_path: &PathBuf,
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

pub fn send_replay_to_server(
    replay_info: &ExtendedGameInformation,
) -> ParserAppResult<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();

    let dto = ReplayReportDto::from(replay_info);

    let request = client
        .post("http://dawnofwar.info/esl/esl-report.php")
        .json(&dto)
        .build()?;

    match client.execute(request) {
        Ok(response) => Ok(response),
        Err(err) => Err(ParserAppError::GenericError(err.to_string())),
    }
}

fn transform_replay_to_base64(replay_file_path: &PathBuf) -> ParserAppResult<String> {
    let bytes = fs::read(replay_file_path)?;

    Ok(base64::encode(bytes))
}

pub fn handle_new_game_event(handle: AppHandle) -> ParserAppResult<()> {
    let Some(main_window_handle) = handle.get_window("main") else {
                    return Err(ParserAppError::GenericError("Could not acquire main window handle. This is unrecoverable".into()));
                };

    let (tx, rx) = std::sync::mpsc::channel();
    // let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let InputFiles {
        replay_file_path,
        logfile_path: _,
    } = get_input_files()?;

    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx).unwrap();

    debouncer.watcher().watch(
        Path::new(replay_file_path.as_os_str()),
        RecursiveMode::Recursive,
    )?;

    for events in rx {
        for _e in events? {
            let mut replay_info = update_game_list()?;

            // Copy replay file to ESL folder
            copy_replay_file(&replay_file_path, &replay_info)?;

            replay_info.replay = transform_replay_to_base64(&replay_file_path).ok();

            let server_response = send_replay_to_server(&replay_info)?;

            replay_info.replay = None;

            replay_info.status = Some(server_response.status().is_success());

            let json = serde_json::to_string_pretty(&replay_info)?;
            main_window_handle.emit_all("new-game", json)?;
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

        let replay_info = update_game_list();
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
    fn can_send_json_to_server() {
        let replay_info = ExtendedGameInformation {
            dev: Some(true),
            replay: Some("ABC".into()),
            status: Some(true),
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

        let res = send_replay_to_server(&replay_info);
        assert!(res.unwrap().status().is_success());
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
