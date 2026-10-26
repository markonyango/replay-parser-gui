#![allow(unused)]

use color_eyre::{eyre::eyre, Help, Report, Result};
use encoding_rs_io::DecodeReaderBytesBuilder;
use lazy_static::lazy_static;
use parser_lib::{
    actions::{Action, Command},
    blueprints::ResolvedBlueprint,
    chunky::{Chunk, Game as ReplayGame, Map, Player},
    message::Message,
    replay::ReplayInfo,
};
use regex::{Captures, Regex, RegexSet};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::Emitter;
use tauri::{Manager, WebviewWindow, Window};
use thiserror::Error;
use tracing::{debug, error, info};

use crate::core::error::{ParserAppError, UploadError};
use crate::core::transformer::transform_player_information;

use super::{
    error::ParserAppResult,
    logfile::LogfileGameInfo,
    player_info::{ExtendedPlayerInformation, LogfilePlayerInfo},
    replay_reporter_dto::ReplayReportDto,
};

const MAP_PATH_PREFIX: &str = "DATA:maps\\pvp\\";
const REPORT_URL: &str = "https://dow2elite.com/esl-report.php";
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(60);

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
    #[serde(serialize_with = "serialize_map_for_frontend")]
    pub map: Map,
    pub players: Vec<ExtendedPlayerInformation>,
    pub messages: Vec<Message>,
    #[serde(serialize_with = "serialize_actions_for_frontend")]
    pub actions: Vec<Action>,
    pub aborted: bool,
    pub frames: usize,
    pub ended_at: String,
    pub status: UploadStatus,
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

/// What the frontend stores per action. The upload sends the full `Action` via `ReplayReportDto`.
///
/// This is an allowlist on purpose: a new `parser_lib` field only reaches store.json once added here.
#[derive(Serialize)]
struct ActionView<'a> {
    relic_id: u64,
    name: &'a str,
    tick: u32,
    command: Option<CommandView<'a>>,
    command_error: Option<&'a str>,
}

impl<'a> From<&'a Action> for ActionView<'a> {
    fn from(action: &'a Action) -> Self {
        Self {
            relic_id: action.relic_id,
            name: &action.player,
            tick: action.tick,
            command: action.command.as_ref().map(CommandView::from),
            command_error: action.command_error.as_deref(),
        }
    }
}

/// `Command` without the raw context, receivers and custom payload bytes.
#[derive(Serialize)]
struct CommandView<'a> {
    opcode: u8,
    opcode_name: Option<&'static str>,
    sender_flags: u8,
    sender_slot: u8,
    process_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    blueprint: Option<&'a ResolvedBlueprint>,
}

impl<'a> From<&'a Command> for CommandView<'a> {
    fn from(command: &'a Command) -> Self {
        Self {
            opcode: command.opcode.value(),
            opcode_name: command.opcode.name(),
            sender_flags: command.sender_flags,
            sender_slot: command.sender_slot,
            process_type: command.process_type,
            blueprint: command.blueprint.as_ref(),
        }
    }
}

fn serialize_actions_for_frontend<S: Serializer>(
    actions: &[Action],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(actions.iter().map(ActionView::from))
}

/// The map's short name, e.g. `6p_calderis` for `DATA:maps\\pvp\\6p_calderis`.
pub fn map_slug(map: &Map) -> String {
    map.path.replace(MAP_PATH_PREFIX, "")
}

/// Serializes the map as parsed, plus its `slug`, so the frontend need not derive it.
fn serialize_map_for_frontend<S: Serializer>(map: &Map, serializer: S) -> Result<S::Ok, S::Error> {
    let mut value = serde_json::to_value(map).map_err(serde::ser::Error::custom)?;
    if let Some(fields) = value.as_object_mut() {
        fields.insert("slug".into(), map_slug(map).into());
    }
    value.serialize(serializer)
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
            status: UploadStatus::Pending,
            dev: None,
            replay: None,
            game: match parsed_replay.game {
                parser_lib::chunky::Chunk::Game(ref game) => GameInfo::from(game.clone()),
                _ => GameInfo::default(),
            },
        };

        res
    }

    pub fn send_replay_to_server(&mut self, replay_file_path: &Path) -> ParserAppResult<()> {
        self.send_replay_to(REPORT_URL, replay_file_path)
    }

    fn send_replay_to(&mut self, url: &str, replay_file_path: &Path) -> ParserAppResult<()> {
        // Without a readable replay there is nothing to report, so this stays a hard error.
        self.transform_replay_to_base64(replay_file_path)?;

        let outcome = reqwest::blocking::Client::builder()
            .timeout(UPLOAD_TIMEOUT)
            .build()
            .map_err(UploadError::Transport)
            .and_then(|client| self.upload(&client, url));

        self.status = match outcome {
            Ok(status) => status,
            Err(err) => {
                error!("Replay upload failed: {:?}", err);
                UploadStatus::Failed {
                    reason: err.to_string(),
                }
            }
        };

        // Never hand the base64 payload on to the frontend, whatever the outcome.
        self.replay = None;

        Ok(())
    }

    fn upload(
        &self,
        client: &reqwest::blocking::Client,
        url: &str,
    ) -> Result<UploadStatus, UploadError> {
        let response = client.post(url).json(&ReplayReportDto::from(self)).send()?;
        let http_status = response.status();

        let body: StatusResponse = response
            .json()
            .map_err(|e| UploadError::BadResponse(http_status, e.to_string()))?;

        debug!("The response message from the server: {:?}", body);

        match (body.response, body.error) {
            (Some(message), _) if http_status.is_success() => {
                Ok(UploadStatus::Accepted { message })
            }
            (_, Some(message)) => Ok(UploadStatus::Rejected { message }),
            _ => Err(UploadError::BadResponse(
                http_status,
                "response carried neither a message nor an error".into(),
            )),
        }
    }

    fn transform_replay_to_base64(&mut self, replay_file_path: &Path) -> ParserAppResult<()> {
        let bytes = fs::read(replay_file_path).map_err(|source| ParserAppError::ReplayRead {
            path: replay_file_path.to_path_buf(),
            source,
        })?;

        self.replay = Some(base64::encode(bytes));

        Ok(())
    }
}

/// Outcome of reporting a replay, sent to the frontend as `{"kind": …, …}`.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UploadStatus {
    #[default]
    Pending,
    /// The server accepted the replay.
    Accepted { message: String },
    /// The server answered but refused the replay.
    Rejected { message: String },
    /// The server could not be reached or gave an unusable answer.
    Failed { reason: String },
}

impl UploadStatus {
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted { .. })
    }

    /// Human-readable detail: the server's message or the failure reason.
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Pending => None,
            Self::Accepted { message } | Self::Rejected { message } => Some(message),
            Self::Failed { reason } => Some(reason),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    response: Option<String>,
    error: Option<String>,
}

/// Copies the replay next to itself as `<match id>_<map>.rec` and returns the copy's path.
pub fn copy_replay_file(
    game_information: &ExtendedGameInformation,
    replay_file_path: &Path,
) -> ParserAppResult<PathBuf> {
    let map_name = map_slug(&game_information.map);
    let target =
        replay_file_path.with_file_name(format!("{}_{}.rec", game_information.id, map_name));

    fs::copy(replay_file_path, &target).map_err(|source| ParserAppError::CopyReplayFile {
        target: target.clone(),
        source,
    })?;

    Ok(target)
}

pub fn notify_main_window(
    game_information: &ExtendedGameInformation,
    main_window_handle: &WebviewWindow,
) -> Result<(), tauri::Error> {
    tracing::debug!("Sending new game to main window");
    main_window_handle.emit("new-game", game_information)
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

        game.send_replay_to(&url, Path::new(FIXTURE_REPLAY))
            .unwrap();

        let request = request.recv().unwrap();
        assert!(request.starts_with("POST /esl-report.php"), "{request}");
        assert!(request.contains(r#""id":"1234""#), "{request}");

        let expected = base64::encode(fs::read(FIXTURE_REPLAY).unwrap());
        assert!(
            request.contains(&format!(r#""replay":"{expected}""#)),
            "request did not carry the base64 replay"
        );

        assert_eq!(
            game.status,
            UploadStatus::Accepted {
                message: "ok".into()
            }
        );
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
            Path::new(FIXTURE_REPLAY),
        )
        .unwrap();

        assert!(
            matches!(game.status, UploadStatus::Failed { .. }),
            "{:?}",
            game.status
        );
    }

    #[test]
    fn records_error_and_drops_payload_when_response_is_not_json() {
        let (url, _request) = serve_once("<html>502 Bad Gateway</html>");
        let mut game = sample_game();

        game.send_replay_to(&url, Path::new(FIXTURE_REPLAY))
            .unwrap();

        assert!(
            matches!(game.status, UploadStatus::Failed { .. }),
            "{:?}",
            game.status
        );
        assert!(game.replay.is_none());
    }

    #[test]
    fn records_server_error_message() {
        let (url, _request) = serve_once(r#"{"error":"bad replay"}"#);
        let mut game = sample_game();

        game.send_replay_to(&url, Path::new(FIXTURE_REPLAY))
            .unwrap();

        assert_eq!(
            game.status,
            UploadStatus::Rejected {
                message: "bad replay".into()
            }
        );
        assert!(game.replay.is_none());
    }

    #[test]
    fn serializes_status_with_kind_tag() {
        let status = |s: UploadStatus| serde_json::to_value(s).unwrap();

        assert_eq!(status(UploadStatus::Pending), json!({ "kind": "pending" }));
        assert_eq!(
            status(UploadStatus::Accepted {
                message: "ok".into()
            }),
            json!({ "kind": "accepted", "message": "ok" })
        );
        assert_eq!(
            status(UploadStatus::Rejected {
                message: "bad replay".into()
            }),
            json!({ "kind": "rejected", "message": "bad replay" })
        );
        assert_eq!(
            status(UploadStatus::Failed {
                reason: "timeout".into()
            }),
            json!({ "kind": "failed", "reason": "timeout" })
        );
    }

    #[test]
    fn frontend_payload_keeps_only_allowlisted_action_fields() {
        let replay = crate::core::parse_replay_file(Path::new(FIXTURE_REPLAY)).unwrap();
        let action = replay
            .actions
            .iter()
            .find(|a| a.command.is_some())
            .expect("fixture has a decoded command")
            .clone();
        let game = ExtendedGameInformation {
            actions: vec![action],
            ..sample_game()
        };

        let json = serde_json::to_value(&game).unwrap();
        let action = &json["actions"][0];
        let keys = |v: &serde_json::Value| {
            let mut keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
            keys.sort();
            keys
        };

        assert_eq!(
            keys(action),
            ["command", "command_error", "name", "relic_id", "tick"]
        );
        let mut command_keys = keys(&action["command"]);
        command_keys.retain(|k| k != "blueprint");
        assert_eq!(
            command_keys,
            [
                "opcode",
                "opcode_name",
                "process_type",
                "sender_flags",
                "sender_slot"
            ]
        );
        // The upload still carries the raw bytes the frontend no longer gets.
        let upload = serde_json::to_value(ReplayReportDto::from(&game)).unwrap();
        assert!(!upload["actions"][0]["data"].as_array().unwrap().is_empty());
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("parser_game_{}_{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn map_slug_strips_the_pvp_directory() {
        let mut map = Map::default();
        map.path = "DATA:maps\\pvp\\6p_calderis".into();
        assert_eq!(map_slug(&map), "6p_calderis");

        map.path = "custom".into();
        assert_eq!(map_slug(&map), "custom");
    }

    #[test]
    fn serialized_map_carries_its_slug() {
        let mut game = sample_game();
        game.map.path = "DATA:maps\\pvp\\6p_calderis".into();

        let json = serde_json::to_value(&game).unwrap();

        assert_eq!(json["map"]["slug"], "6p_calderis");
        assert_eq!(json["map"]["path"], "DATA:maps\\pvp\\6p_calderis");
    }

    #[test]
    fn copies_replay_under_match_id_and_map_name() {
        let dir = temp_dir("copy_ok");
        let replay = dir.join("temp.rec");
        fs::write(&replay, b"replay bytes").unwrap();
        let mut game = sample_game();
        game.map.path = "DATA:maps\\pvp\\6p_calderis".into();

        let copy = copy_replay_file(&game, &replay).unwrap();

        assert_eq!(copy, dir.join("1234_6p_calderis.rec"));
        assert_eq!(fs::read(copy).unwrap(), b"replay bytes");
    }

    #[test]
    fn copy_error_keeps_the_io_cause() {
        let dir = temp_dir("copy_missing");

        let err = copy_replay_file(&sample_game(), &dir.join("temp.rec")).unwrap_err();

        let ParserAppError::CopyReplayFile { source, .. } = err else {
            panic!("unexpected error: {err:?}");
        };
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn fails_when_replay_file_is_missing() {
        let mut game = sample_game();

        let res = game.send_replay_to(
            "http://127.0.0.1:9/esl-report.php",
            Path::new("does-not-exist.rec"),
        );

        assert!(res.is_err());
    }
}
