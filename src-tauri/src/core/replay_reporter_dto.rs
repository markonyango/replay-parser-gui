use chrono::prelude::*;
use parser_lib::{actions::Action, message::Message};
use serde::Serialize;

use super::{
    game::ExtendedGameInformation,
    player_info::{ExtendedPlayerInformation, LogfilePlayerStatus},
};

#[derive(Debug, Serialize)]
pub struct ReplayReportDto {
    aborted: bool,
    actions: Vec<Action>,
    dev: bool,
    id: String,
    map: String,
    reporter: ReplayReportReporterDto,
    replay: String,
    mod_version: usize,
    ranked: bool,
    league: bool,
    frames: usize,
    ticks: usize,
    players: Vec<ReplayReporterPlayerDto>,
    messages: Vec<ReplayReporterMessageDto>,
    // observers: Vec<>
    winner: usize,
}

#[derive(Debug, Serialize)]
pub struct ReplayReportReporterDto {
    date: String,
    version: String,
}

#[derive(Debug, Serialize)]
pub struct ReplayReporterPlayerDto {
    relic_id: usize,
    hero: usize,
    race: usize,
    name: String,
    steam_id: usize,
    team: usize,
    sim_id: usize,
    slot: usize,
}

#[derive(Debug, Serialize)]
pub struct ReplayReporterMessageDto {
    receiver: String,
    sender: String,
    body: String,
    tick: usize,
    player_id: usize,
}

impl ReplayReporterPlayerDto {
    pub fn from(player: &ExtendedPlayerInformation) -> Self {
        Self {
            relic_id: player.relic_id as usize,
            hero: player.hero as usize,
            race: player.race,
            name: player.name.clone(),
            steam_id: player.steam_id,
            team: player.team as usize,
            sim_id: player.sim_id,
            slot: player.slot,
        }
    }
}

impl ReplayReporterMessageDto {
    pub fn from(message: &Message) -> Self {
        Self {
            receiver: message.receiver.clone(),
            sender: message.sender.clone(),
            body: message.body.clone(),
            tick: message.tick as usize,
            player_id: message.player_id as usize,
        }
    }
}

impl ReplayReportDto {
    pub fn from(replay: &ExtendedGameInformation) -> Self {
        let winner = replay
            .players
            .iter()
            .find(|player| matches!(player.status, LogfilePlayerStatus::Won))
            .map(|player| player.team)
            .unwrap_or(0);

        Self {
            aborted: replay.aborted,
            actions: replay
                .actions
                .iter()
                .map(|action| Action {
                    player: action.player.clone(),
                    relic_id: action.relic_id,
                    tick: action.tick,
                    data: action.data.clone(),
                })
                .collect(),
            dev: if let Some(dev) = replay.dev {
                dev
            } else {
                false
            },
            id: replay.id.to_string(),
            map: replay.map.path.clone().replace("DATA:maps\\pvp\\", ""),
            reporter: ReplayReportReporterDto {
                date: chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                version: "1.0.3".into(),
            },
            replay: if let Some(replay_string) = replay.replay.clone() {
                replay_string
            } else {
                "".into()
            },
            mod_version: replay.mod_version,
            ranked: false,
            league: false,
            frames: replay.frames,
            ticks: replay.ticks,
            players: replay
                .players
                .iter()
                .map(ReplayReporterPlayerDto::from)
                .collect(),
            messages: replay
                .messages
                .iter()
                .map(ReplayReporterMessageDto::from)
                .collect(),
            winner: winner as usize,
        }
    }
}
