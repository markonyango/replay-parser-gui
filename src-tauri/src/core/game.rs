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
use std::{collections::HashMap, error, fs::File, io::Read, path::Path};
use thiserror::Error;

use crate::core::error::ParserAppError;

use super::{
    error::ParserAppResult,
    player_info::{ExtendedPlayerInformation, LogfilePlayerInfo},
};

const MATCH_BLOCK_PATTERNS: [&str; 9] = [
    r"Match Started - \[\d+:(.+) /steam/(\d+)\], slot =\D+(\d)",
    r"Beginning mission (.+) \((\d) Humans, (\d) Computers\)",
    r"GAME -- Frame",
    r"SimID:(\d+), raceID:(\d+), teamID:(\d+), uid:\d+:(\d+), result:\d{1}:(.+)",
    r"SimID:(\d+), raceID:(\d+), teamID:(\d+), uid:\[\d+:(.+)\]",
    r"ReportSimStats - storing simulation results for match \d:(\d+)",
    r"Ending mission - '(\D+)'",
    r"Game Over at frame (\d+)",
    r"pid 0:(\d+), /steam/(\d+)",
];

lazy_static! {
    static ref LOGFILE_FILTER_REGEXP: RegexSet = regex::RegexSet::new([
        r"Beginning mission",
        r"Ending mission",
        r"GAME -- Frame",
        r"ReportSimStats",
        r"ReportMatchStatsForPVP - SimID",
        r"PlayerInfo",
        r"Match Started",
        r"MOD -- Game Over at frame",
        r"LoadArbitrator::UpdateLoadProgress - info",
    ])
    .unwrap();
    static ref GAME_START_REGEXP: RegexSet = RegexSet::new(MATCH_BLOCK_PATTERNS).unwrap();
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct LogfileGameInfo {
    pub aborted: bool,
    pub id: usize,
    pub map: String,
    pub frames: usize,
    pub ended_at: String,
    pub winner: u8,
    pub players: Vec<LogfilePlayerInfo>,
    pub complete: bool,
}

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

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct LogfileGameList {
    logfile_content: Vec<String>,
    pub games: Vec<LogfileGameInfo>,
}

#[derive(Debug)]
struct MatchGroup<'a> {
    index: usize,
    captures: Captures<'a>,
}

impl LogfileGameInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_complete(&self) -> bool {
        self.complete
    }
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

#[derive(Debug, Error)]
#[error("{0}")]
pub struct LogfileNotFoundError(String);

#[derive(Debug, Default)]
pub struct SteamIdMap {
    relic_id: usize,
    slot: usize,
    uid: String, // Game internal user id per player that is assigned when the match starts. Will be used to identify dropped players.
}

impl LogfileGameList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_logfile(&mut self, logfilepath: &Path) -> ParserAppResult<()> {
        if !logfilepath.exists() {
            return Err(ParserAppError::LogfileNotFoundError);
        }

        // Rust can not directly read from this file since it is not UTF-8 encoded
        let mut logfile = File::open(logfilepath)?;
        let mut reader = DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::UTF_8))
            .build(logfile);

        let mut buffer = vec![];

        reader.read_to_end(&mut buffer)?;

        let result = String::from_utf8(buffer)?
            .lines()
            .map(|line| line.to_string())
            .filter(|line| contains_desired_content(line))
            .collect::<Vec<_>>();

        self.logfile_content = result;

        Ok(())
    }

    pub fn parse(&mut self) -> ParserAppResult<()> {
        let regexes = GAME_START_REGEXP
            .patterns()
            .iter()
            .map(|pat| Regex::new(pat).unwrap())
            .collect::<Vec<_>>();

        let mut match_header: HashMap<usize, SteamIdMap> = HashMap::new();

        for line in self.logfile_content.iter() {
            let matches = GAME_START_REGEXP.matches(line);
            let match_captures = matches
                .into_iter()
                .map(|match_index| {
                    let pat = &regexes[match_index];
                    let captures = pat.captures(line).unwrap();

                    MatchGroup {
                        index: match_index,
                        captures,
                    }
                })
                .collect::<Vec<_>>();

            if match_captures.len() != 1 {
                return Err(ParserAppError::LogfileParseError(
                    "Found more than 1 game to parse".into(),
                ));
            }

            let match_group = &match_captures[0];

            match match_group.index {
                0 => {
                    let Some(uid) = match_group.captures.get(1) else {
                        return Err(ParserAppError::LogfileParseError("Could not parse user id from match header block".into()));
                    };

                    let Some(steam_id) = match_group.captures.get(2) else {
                        return Err(ParserAppError::LogfileParseError("Could not parse steam id from match header block".into()));
                    };

                    let Some(slot) = match_group.captures.get(3) else {
                        return Err(ParserAppError::LogfileParseError("Could not parse slot number from match header block".into()));
                    };

                    let uid = uid.as_str().into();
                    let steam_id = steam_id.as_str().parse::<usize>().unwrap();
                    let slot = slot.as_str().parse::<usize>().unwrap();

                    match_header.insert(
                        steam_id,
                        SteamIdMap {
                            relic_id: 0,
                            slot,
                            uid,
                        },
                    );
                }
                1 => {
                    // Is there any game in the list to begin with
                    if let Some(last_game) = self.games.last_mut() {
                        // We can't know whether game block was created with capture index 0 or 1
                        // -> check if last game block is complete, i.e. Ending mission was read
                        match last_game.block_complete() {
                            true => {
                                self.games.push(LogfileGameInfo::new());
                            }
                            false => (),
                        }
                    } else {
                        self.games.push(LogfileGameInfo::new());
                    }

                    // At this point there should be a game in the list
                    let Some(map) = match_group.captures.get(1) else {
                        return Err(ParserAppError::LogfileParseError("Could not parse map from logfile".into()));
                    };

                    let len = self.games.len() - 1;
                    self.games[len].map = map.as_str().to_string();
                }
                2 => (),
                3 => {
                    if let Some(last_game) = self.games.last_mut() {
                        let mut player = LogfilePlayerInfo::new();
                        player.parse(&match_group.captures, false);

                        // Add slot number and steam id from hashmap
                        let steam_id = match_header.iter().find_map(|(key, val)| {
                            if val.relic_id == player.relic_id {
                                Some(key)
                            } else {
                                None
                            }
                        });
                        if let Some(steam_id) = steam_id {
                            player.steam_id = *steam_id;
                            player.slot = match_header.get(steam_id).unwrap().slot;
                        }

                        if let Some(last_game) = self.games.last_mut() {
                            last_game.players.push(player);
                        }
                    }
                }
                4 => {
                    if let Some(last_game) = self.games.last_mut() {
                        let mut player = LogfilePlayerInfo::new();
                        player.parse(&match_group.captures, true);

                        // This match result line does not contain the players relic id but his game internal user id
                        let uid = match_group.captures.get(4).unwrap().as_str().to_string();

                        // Add slot number and steam id from hashmap
                        let steam_id = match_header.iter().find_map(|(key, val)| {
                            if val.uid == uid {
                                Some(key)
                            } else {
                                None
                            }
                        });
                        if let Some(steam_id) = steam_id {
                            player.steam_id = *steam_id;
                            player.slot = match_header.get(steam_id).unwrap().slot;
                            player.relic_id = match_header.get(steam_id).unwrap().relic_id;
                        }

                        if let Some(last_game) = self.games.last_mut() {
                            last_game.players.push(player);
                        }
                    }
                }
                5 => {
                    if let Some(last_game) = self.games.last_mut() {
                        let Some(match_relic_id) = match_group.captures.get(1) else {
                            return Err(ParserAppError::LogfileParseError("Could not extract match relic id from logfile".into()));
                        };

                        let Ok(match_relic_id) = match_relic_id.as_str().parse::<usize>() else {
                            return Err(ParserAppError::LogfileParseError("Could not parse match relic id in logfile".into()));
                        };

                        last_game.id = match_relic_id;
                    }
                }
                6 => {
                    if let Some(last_game) = self.games.last_mut() {
                        // Get game ending status
                        match match_group.captures.get(1) {
                            Some(capture) => match capture.as_str() {
                                "Game over" => {
                                    last_game.aborted = false;
                                    last_game.complete = true;
                                }
                                "Abort" => {
                                    last_game.aborted = true;
                                    last_game.complete = true;
                                }
                                // Unknown status - defaulting to a cancelled and complete game
                                unknown_status => {
                                    last_game.aborted = true;
                                    last_game.complete = true;
                                }
                            },
                            // Could not find the game ending status information - defaulting to a
                            // cancelled and complete game
                            None => {
                                last_game.aborted = true;
                                last_game.complete = true;
                            }
                        }
                    }
                }
                7 => {
                    if let Some(last_game) = self.games.last_mut() {
                        let Some(frames) = match_group.captures.get(1) else {
                            return Err(ParserAppError::LogfileParseError("Could not extract number of frames from logfile".into()));
                        };

                        let Ok(frames) = frames.as_str().parse::<usize>() else {
                            return Err(ParserAppError::LogfileParseError("Could not parse number of frames from logfile".into()));
                        };

                        last_game.frames = frames;
                    }
                }
                8 => {
                    let Some(relic_id) = match_group.captures.get(1) else {
                        return Err(ParserAppError::LogfileParseError("Could not parse relic id from log file".into()));
                    };

                    let Some(steam_id) = match_group.captures.get(2) else {
                        return Err(ParserAppError::LogfileParseError("Could not parse steam if from log file".into()));
                    };

                    let steam_id = steam_id.as_str().parse::<usize>().unwrap();
                    let relic_id = relic_id.as_str().parse::<usize>().unwrap();

                    if let Some(info) = match_header.get_mut(&steam_id) {
                        info.relic_id = relic_id;
                    } else {
                        match_header.insert(
                            steam_id,
                            SteamIdMap {
                                relic_id,
                                ..Default::default()
                            },
                        );
                    }
                }
                capture_group => {
                    return Err(ParserAppError::LogfileParseError(format!(
                        "RegEx error while parsing logfile: {}",
                        capture_group
                    )))
                }
            }
        }

        Ok(())
    }
}

impl ExtendedGameInformation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(parsed_replay: ReplayInfo, parsed_logfile_game: &LogfileGameInfo) -> Self {
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

        ExtendedGameInformation {
            aborted: parsed_logfile_game.aborted,
            id: parsed_logfile_game.id,
            map: map.unwrap_or_default(),
            frames: parsed_logfile_game.frames,
            ended_at: parsed_replay.date.clone(),
            players: players_with_extended_information,
            messages: parsed_replay.messages,
            actions,
            name: parsed_replay.name,
            mod_chksum: parsed_replay.mod_chksum as usize,
            mod_version: parsed_replay.mod_version as usize,
            md5: parsed_replay.md5,
            date: parsed_replay.date.clone(),
            ticks: parsed_replay.ticks as usize,
            status: "".into(),
            dev: None,
            replay: None,
            game,
        }
    }
}

fn contains_desired_content(string: &str) -> bool {
    LOGFILE_FILTER_REGEXP
        .matches(string)
        .into_iter()
        .next()
        .is_some()
}

#[cfg(test)]
mod tests {

    use crate::core::player_info::LogfilePlayerStatus;

    use super::*;

    #[test]
    fn read_match_from_logfile() {
        let logfilepath = Path::new("warnings.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();

        assert_eq!(game_list.logfile_content.len(), 240);
    }

    #[test]
    fn parse_two_matches_from_logfile() {
        let logfilepath = Path::new("warnings2.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();
        game_list.parse().unwrap();

        assert_eq!(game_list.games.len(), 2);
        assert_eq!(game_list.games[0].players.len(), 2);
        assert_eq!(game_list.games[1].players.len(), 6);

        assert_eq!(game_list.games[0].players[0].get_race(), 0);
        assert_eq!(game_list.games[0].players[0].get_team_id(), 0);
        assert_eq!(game_list.games[0].players[0].get_sim_id(), 1000);
        assert_eq!(
            *game_list.games[0].players[0].get_status(),
            LogfilePlayerStatus::Conceded
        );

        assert!(!game_list.games[0].aborted);
        assert_eq!(game_list.games[0].id, 54926186);
        assert_eq!(game_list.games[0].map, "2p_calderisrefinery");
    }

    #[test]
    fn parse_match_with_dropped_player() {
        let logfilepath = Path::new("warnings.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();
        game_list.parse().unwrap();

        assert_eq!(game_list.games.len(), 6);
        assert_eq!(
            *game_list.games[5].players[0].get_status(),
            LogfilePlayerStatus::Won
        );
        assert_eq!(
            *game_list.games[5].players[1].get_status(),
            LogfilePlayerStatus::Won
        );
        assert_eq!(
            *game_list.games[5].players[2].get_status(),
            LogfilePlayerStatus::Won
        );
        assert_eq!(
            *game_list.games[5].players[3].get_status(),
            LogfilePlayerStatus::Dropped
        );
        assert_eq!(
            *game_list.games[5].players[4].get_status(),
            LogfilePlayerStatus::Conceded
        );
        assert_eq!(
            *game_list.games[5].players[5].get_status(),
            LogfilePlayerStatus::Conceded
        );
    }

    #[test]
    fn parse_match_with_observer() {
        let logfilepath = Path::new("warnings_with_observer.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();
        game_list.parse().unwrap();

        assert_eq!(game_list.games.len(), 4);
        assert_eq!(game_list.games[3].players.len(), 6);
        assert_eq!(
            *game_list.games[3].players[0].get_status(),
            LogfilePlayerStatus::Won
        );
        assert_eq!(
            *game_list.games[3].players[1].get_status(),
            LogfilePlayerStatus::Won
        );
        assert_eq!(
            *game_list.games[3].players[2].get_status(),
            LogfilePlayerStatus::Won
        );
        assert_eq!(
            *game_list.games[3].players[3].get_status(),
            LogfilePlayerStatus::Conceded
        );
        assert_eq!(
            *game_list.games[3].players[4].get_status(),
            LogfilePlayerStatus::Conceded
        );
        assert_eq!(
            *game_list.games[3].players[5].get_status(),
            LogfilePlayerStatus::Conceded
        );
    }

    #[test]
    fn can_parse_relic_and_steam_id() {
        let logfilepath = Path::new("warnings3.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();
        game_list.parse().unwrap();

        assert!(game_list.games[0].players[0].relic_id != 0);
        assert!(game_list.games[0].players[0].steam_id != 0);
        assert_eq!(game_list.games[0].players[0].slot, 0);
        assert!(game_list.games[0].players[1].relic_id != 0);
        assert!(game_list.games[0].players[1].steam_id != 0);
        assert_eq!(game_list.games[0].players[1].slot, 1);
    }
}
