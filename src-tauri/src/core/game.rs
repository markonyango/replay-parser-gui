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
use tauri::{Window, Manager};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tracing::{error, info};

use crate::core::error::ParserAppError;

use super::{
    error::ParserAppResult,
    logfile::LogfileGameInfo,
    player_info::{ExtendedPlayerInformation, LogfilePlayerInfo},
    replay_reporter_dto::ReplayReportDto,
};

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

impl GameInfo {
    pub fn from(game: ReplayGame) -> Self {
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(
        &mut self,
        parsed_replay: ReplayInfo,
        parsed_logfile_game: &LogfileGameInfo,
    ) -> &mut Self {
        let players_with_extended_information = parsed_replay
            .players
            .iter()
            .enumerate()
            .map(|(index, replayfile_player)| {
                let logfile_player = parsed_logfile_game.players.get(index);

                let _player = match replayfile_player {
                    parser_lib::chunky::Chunk::Player(player) => Some(player),
                    _ => None,
                };

                match (logfile_player, _player) {
                    (Some(logfile_player), Some(player)) => {
                        ExtendedPlayerInformation::from(logfile_player, player)
                    }
                    _ => ExtendedPlayerInformation::default(),
                }
            })
            .collect::<Vec<_>>();

        let map = match parsed_replay.map {
            parser_lib::chunky::Chunk::Map(map) => Some(map),
            _ => None,
        };

        let game = match parsed_replay.game {
            parser_lib::chunky::Chunk::Game(game) => GameInfo::from(game),
            _ => GameInfo::default(),
        };

        let actions = parsed_replay
            .actions
            .iter()
            .filter_map(|action| {
                action.data.get(3).and_then(|&id| {
                    let _id = (id - 0xE8) as usize + 1000;
                    let _player = players_with_extended_information
                        .iter()
                        .find(|&player| player.sim_id == _id);

                    _player.map(|p| Action {
                        player: p.name.clone(),
                        relic_id: p.relic_id,
                        tick: action.tick,
                        data: action.data.clone(),
                    })
                })
            })
            .collect::<Vec<_>>();

        self.aborted = parsed_logfile_game.aborted;
        self.id = parsed_logfile_game.id;
        self.map = map.unwrap_or_default();
        self.frames = parsed_logfile_game.frames;
        self.ended_at = parsed_replay.date.clone();
        self.players = players_with_extended_information;
        self.messages = parsed_replay.messages;
        self.actions = actions;
        self.name = parsed_replay.name;
        self.mod_chksum = parsed_replay.mod_chksum as usize;
        self.mod_version = parsed_replay.mod_version as usize;
        self.md5 = parsed_replay.md5;
        self.date = parsed_replay.date.clone();
        self.ticks = parsed_replay.ticks as usize;
        self.status = "".into();
        self.dev = None;
        self.replay = None;
        self.game = game;

        self
    }

    pub fn copy_replay_file(&mut self, replay_file_path: &PathBuf) -> ParserAppResult<&mut Self> {
        let mut file_name = replay_file_path.clone();
        let map_name = self.map.path.replace("DATA:maps\\pvp\\", "");

        file_name.set_file_name(format!("{}_{}.rec", self.id, map_name));

        let Ok(_) = fs::copy(replay_file_path, file_name) else {
             return Err(ParserAppError::ParserLibError("Could not copy replay file".into()));
        };

        Ok(self)
    }

    pub fn send_replay_to_server(&mut self) -> ParserAppResult<&mut Self> {
        let client = reqwest::blocking::Client::new();

        let dto = ReplayReportDto::from(self);

        let request = client
            .post("http://dawnofwar.info/esl/esl-report.php")
            .json(&dto)
            .build()?;

        match client.execute(request) {
            Ok(response) => {
                let response_body = response.text();
                info!("The response message from the server: {:?}", response_body);
                match response_body {
                    Ok(body) if body.contains("error") => self.status = body,
                    Ok(body) if !body.contains("error") => self.status = body,
                    Ok(_) => self.status = json!({ "response": "ok" }).to_string(),
                    Err(error) => self.status = error.to_string(),
                };

                self.replay = None;
            }
            Err(err) => {
                error!("{:?}", err.to_string());
                self.status = json!({ "error": err.to_string()}).to_string();
            }
        }

        Ok(self)
    }

    pub fn transform_replay_to_base64(&mut self, replay_file_path: &PathBuf) -> ParserAppResult<&mut Self> {
        let bytes = fs::read(replay_file_path)?;

        self.replay = Some(base64::encode(bytes));

        Ok(self)
    }

    pub fn notify_main_window(&mut self, main_window_handle: &Window) -> Result<(), tauri::Error> {
        let json = serde_json::to_string_pretty(self)?;
        main_window_handle.emit_all("new-game", json)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::game;

    use super::*;

    #[test]
    fn can_send_replay_to_server() {
        let mut replay_info = ExtendedGameInformation {
            dev: Some(true),
            replay: Some("ABC".into()),
            status: Default::default(),
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

        let res = replay_info.send_replay_to_server();
        assert!(res.is_ok());
    }
}

