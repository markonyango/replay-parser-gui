#![allow(unused)]

use color_eyre::Result;
use lazy_static::lazy_static;
use parser_lib::chunky::{self, Chunk};
use regex::{Captures, Match, Regex};
use serde::{Deserialize, Serialize};

lazy_static! {
    pub static ref PLAYER_INFO_REG_EXP: Regex =
        Regex::new(r"SimID:(\d+), raceID:(\d+), teamID:(\d+), uid:\d+:(\d+), result:\d{1}:(.+)")
            .unwrap();
    pub static ref PLAYER_INFO_REGEXP_ALT: Regex =
        Regex::new(r"SimID:(\d+), raceID:(\d+), teamID:(\d+), uid:\[\d+:(.+)\]").unwrap();
}

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub enum LogfilePlayerStatus {
    #[default]
    Unknown,
    Won,       // Player won the game
    Conceded,  // Player conceded before VPs reached 0
    Killed,    // Players VPs reached 0
    Playing,   // Player was in a match with somebody that either lost connection or rage quitted
    Outofsync, // Player probably rage quitted or lost connection and caused an "Out of Sync" error
    Dropped,   // Player disconnected from the game without causing an "Out of Sync" error
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct LogfilePlayerInfo {
    pub sim_id: usize,
    pub race: usize,
    pub team_id: usize,
    pub relic_id: usize,
    pub steam_id: usize,
    pub slot: usize,
    pub status: LogfilePlayerStatus,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct ExtendedPlayerInformation {
    pub slot: usize,
    pub steam_id: usize,
    pub sim_id: usize,
    pub status: LogfilePlayerStatus,
    pub name: String,
    pub kind: u32,
    pub team: u32,
    pub race: usize,
    pub relic_id: u64,
    pub rank: u32,
    pub cpu: u32,
    pub hero: u32,
    pub primary_color: u8,
    pub secondary_color: u8,
    pub trim_color: u8,
    pub accessory_color: u8,
    pub skin_path: String,
    pub skin_name: String,
    pub id: u8,
}

impl LogfilePlayerInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_sim_id(&self) -> usize {
        self.sim_id
    }

    pub fn get_race(&self) -> usize {
        self.race
    }

    pub fn get_team_id(&self) -> usize {
        self.team_id
    }

    pub fn get_status(&self) -> &LogfilePlayerStatus {
        &self.status
    }

    pub fn parse(&mut self, captures: &Captures, alt: bool) {
        if alt {
            self.sim_id = captures.get(1).map_or(0, convert_match_to_int);
            self.race = captures.get(2).map_or(0, convert_match_to_int);
            self.team_id = captures.get(3).map_or(0, convert_match_to_int);
            self.status = LogfilePlayerStatus::Dropped;
        } else {
            self.sim_id = captures.get(1).map_or(0, convert_match_to_int);
            self.race = captures.get(2).map_or(0, convert_match_to_int);
            self.team_id = captures.get(3).map_or(0, convert_match_to_int);
            self.relic_id = captures.get(4).map_or(0, convert_match_to_int);
            self.status = captures
                .get(5)
                .map_or(LogfilePlayerStatus::Unknown, convert_match_to_status);
        }
    }
}

impl ExtendedPlayerInformation {
    pub fn from(logfile_player: &LogfilePlayerInfo, replayfile_player: &chunky::Player) -> Self {
        Self {
            slot: logfile_player.slot,
            steam_id: logfile_player.steam_id,
            sim_id: logfile_player.get_sim_id(),
            status: logfile_player.get_status().clone(),
            name: replayfile_player.name.clone(),
            kind: replayfile_player.kind,
            team: replayfile_player.team,
            race: logfile_player.race,
            relic_id: replayfile_player.relic_id,
            rank: replayfile_player.rank,
            cpu: replayfile_player.cpu,
            hero: replayfile_player.hero,
            primary_color: replayfile_player.primary_color,
            secondary_color: replayfile_player.secondary_color,
            trim_color: replayfile_player.trim_color,
            accessory_color: replayfile_player.accessory_color,
            skin_path: replayfile_player.skin_path.clone(),
            skin_name: replayfile_player.skin_name.clone(),
            id: replayfile_player.id,
        }
    }
}

fn convert_match_to_int(capture: Match) -> usize {
    if let Ok(int) = capture.as_str().parse::<usize>() {
        return int;
    }

    0
}

fn convert_match_to_status(status: Match) -> LogfilePlayerStatus {
    match status.as_str() {
        "PS_KILLED" => LogfilePlayerStatus::Killed,
        "PS_WON" => LogfilePlayerStatus::Won,
        "PS_CONCEDED" => LogfilePlayerStatus::Conceded,
        "PS_PLAYING" => LogfilePlayerStatus::Playing,
        "PS_OUTOFSYNC" => LogfilePlayerStatus::Outofsync,
        _ => LogfilePlayerStatus::Unknown,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_player_info_from_string() {
        let line = "13:50:45.82    PlayerInfo - SimID:1001, raceID:4, teamID:0, uid:0:11718717, result:3:PS_KILLED";
        let captures = PLAYER_INFO_REG_EXP.captures(line).unwrap();

        let mut info = LogfilePlayerInfo::new();
        info.parse(&captures, false);

        assert_eq!(info.sim_id, 1001);
        assert_eq!(info.race, 4);
        assert_eq!(info.team_id, 0);
        assert_eq!(info.status, LogfilePlayerStatus::Killed);
    }

    #[test]
    fn get_player_info_from_dropped_player() {
        let line = "16:22:54.78    ReportMatchStatsForPVP - SimID:1000, raceID:4, teamID:1, uid:[00000000:0098c7db], AI player, ignoring";
        let captures = PLAYER_INFO_REGEXP_ALT.captures(line).unwrap();

        let mut info = LogfilePlayerInfo::new();
        info.parse(&captures, true);

        assert_eq!(info.sim_id, 1000);
        assert_eq!(info.race, 4);
        assert_eq!(info.team_id, 1);
        assert_eq!(info.status, LogfilePlayerStatus::Dropped);
    }
}
