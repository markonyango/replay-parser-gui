#![allow(unused)]

use color_eyre::{eyre::eyre, Help, Report, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use lazy_static::lazy_static;
use parser_lib::{
    actions::Action,
    chunky::{Chunk, Game as ReplayGame, Map, Player},
    message::Message,
    replay::ReplayInfo,
};
use regex::{Captures, Regex, RegexSet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use tauri::Emitter;
use tauri::{Manager, WebviewWindow, Window};
use thiserror::Error;
use tracing::{debug, error, info};

use crate::core::error::ParserAppError;
use crate::core::transformer::transform_player_information;

use super::{
    error::ParserAppResult,
    logfile::LogfileGameInfo,
    player_info::{ExtendedPlayerInformation, LogfilePlayerInfo},
    replay_reporter_dto::ReplayReportDto,
};

const REPORT_URL: &str = "https://dow2elite.com/esl-report.php";

#[derive(Debug, Default, Serialize)]
pub struct ExtendedGameInformation {
    pub id: usize,
    pub name: String,
    pub mod_chksum: usize,
    pub mod_version: usize,
    pub md5: String,
    pub date: String,
    pub ticks: usize,
    pub game: GameInfo,
    pub map: Map,
    pub players: Vec<ExtendedPlayerInformation>,
    pub messages: Vec<Message>,
    pub actions: Vec<Action>,
    pub aborted: bool,
    pub frames: usize,
    pub ended_at: String,
    pub status: String,
    pub dev: Option<bool>,
    pub replay: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct GameInfo {
    pub name: String,
    pub mode: String,
    pub resources: String,
    pub locations: String,
    pub victory_points: usize,
}

impl From<ReplayGame> for GameInfo {
    fn from(game: ReplayGame) -> Self {
        Self {
            name: game.name,
            mode: game.mode,
            resources: game.resources,
            locations: game.locations,
            victory_points: game.victory_points as usize,
        }
    }
}

impl ExtendedGameInformation {
    pub fn from(parsed_replay: &ReplayInfo, parsed_logfile_game: &LogfileGameInfo) -> Self {
        let mut res = Self {
            aborted: parsed_logfile_game.aborted,
            id: parsed_logfile_game.id,
            map: match parsed_replay.map {
                parser_lib::chunky::Chunk::Map(ref map) => map.clone(),
                _ => Map::default(),
            },
            frames: parsed_logfile_game.frames,
            ended_at: parsed_replay.date.clone(),
            messages: parsed_replay.messages.clone(),
            actions: parsed_replay.actions.clone(),
            players: transform_player_information(parsed_logfile_game, parsed_replay),
            name: parsed_replay.name.clone(),
            mod_chksum: parsed_replay.mod_chksum as usize,
            mod_version: parsed_replay.mod_version as usize,
            md5: parsed_replay.md5.clone(),
            date: parsed_replay.date.clone(),
            ticks: parsed_replay.ticks as usize,
            status: "".into(),
            dev: None,
            replay: None,
            game: match parsed_replay.game {
                parser_lib::chunky::Chunk::Game(ref game) => GameInfo::from(game.clone()),
                _ => GameInfo::default(),
            },
        };

        res
    }

    pub fn send_replay_to_server(
        &mut self,
        replay_file_path: &PathBuf,
    ) -> ParserAppResult<&mut Self> {
        self.send_replay_to(REPORT_URL, replay_file_path)
    }

    fn send_replay_to(
        &mut self,
        url: &str,
        replay_file_path: &PathBuf,
    ) -> ParserAppResult<&mut Self> {
        let client = reqwest::blocking::Client::new();

        self.transform_replay_to_base64(replay_file_path)?;

        let dto = ReplayReportDto::from(self);

        let request = client.post(url).json(&dto).build()?;

        match client.execute(request) {
            Ok(response) => {
                let response_body: StatusResponse = response.json()?;

                debug!("The response message from the server: {:?}", response_body);

                if let Some(response) = response_body.response {
                    self.status = response;
                } else if let Some(error) = response_body.error {
                    self.status = error;
                }

                self.replay = None;
            }
            Err(err) => {
                error!("{:?}", err.to_string());
                self.status = json!({ "error": err.to_string()}).to_string();
            }
        }

        Ok(self)
    }

    fn transform_replay_to_base64(&mut self, replay_file_path: &PathBuf) -> ParserAppResult<()> {
        let bytes = fs::read(replay_file_path)?;

        self.replay = Some(base64::encode(bytes));

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    response: Option<String>,
    error: Option<String>,
}

pub fn copy_replay_file<'a>(
    game_information: &'a ExtendedGameInformation,
    replay_file_path: &'a PathBuf,
) -> ParserAppResult<()> {
    let mut file_name = replay_file_path.clone();
    let map_name = game_information.map.path.replace("DATA:maps\\pvp\\", "");

    file_name.set_file_name(format!("{}_{}.rec", game_information.id, map_name));

    let Ok(_) = fs::copy(replay_file_path, file_name) else {
        return Err(ParserAppError::ParserLibError(
            "Could not copy replay file".into(),
        ));
    };

    Ok(())
}

pub fn notify_main_window<'a>(
    game_information: &'a ExtendedGameInformation,
    main_window_handle: &'a WebviewWindow,
) -> Result<(), tauri::Error> {
    let json = serde_json::to_string_pretty(game_information)?;
    tracing::debug!("Sending new game to main window");
    main_window_handle.emit("new-game", json)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::mpsc,
        thread,
    };

    use super::*;

    const FIXTURE_REPLAY: &str = "3v3.rec";

    /// Serves exactly one request with `response` and returns what the client sent.
    fn serve_once(response: &'static str) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/esl-report.php", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());

            let mut head = String::new();
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().unwrap();
                }
                head.push_str(&line);
            }

            let mut body = vec![0; content_length];
            reader.read_exact(&mut body).unwrap();
            tx.send(format!("{head}\r\n{}", String::from_utf8_lossy(&body)))
                .unwrap();

            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                response.len()
            )
            .unwrap();
        });

        (url, rx)
    }

    fn sample_game() -> ExtendedGameInformation {
        ExtendedGameInformation {
            id: 1234,
            mod_version: 1234,
            ticks: 123,
            frames: 123,
            dev: Some(true),
            ..Default::default()
        }
    }

    #[test]
    fn sends_replay_to_server_and_stores_response() {
        let (url, request) = serve_once(r#"{"response":"ok"}"#);
        let mut game = sample_game();

        game.send_replay_to(&url, &PathBuf::from(FIXTURE_REPLAY))
            .unwrap();

        let request = request.recv().unwrap();
        assert!(request.starts_with("POST /esl-report.php"), "{request}");
        assert!(request.contains(r#""id":"1234""#), "{request}");

        let expected = base64::encode(fs::read(FIXTURE_REPLAY).unwrap());
        assert!(
            request.contains(&format!(r#""replay":"{expected}""#)),
            "request did not carry the base64 replay"
        );

        assert_eq!(game.status, "ok");
    }

    #[test]
    fn records_error_when_server_is_unreachable() {
        // Bind and drop to get a port nothing listens on.
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let mut game = sample_game();

        game.send_replay_to(
            &format!("http://127.0.0.1:{port}/esl-report.php"),
            &PathBuf::from(FIXTURE_REPLAY),
        )
        .unwrap();

        assert!(game.status.contains("error"), "{}", game.status);
    }

    #[test]
    fn fails_when_replay_file_is_missing() {
        let mut game = sample_game();

        let res = game.send_replay_to(
            "http://127.0.0.1:9/esl-report.php",
            &PathBuf::from("does-not-exist.rec"),
        );

        assert!(res.is_err());
    }
}
