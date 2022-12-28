use std::{collections::HashMap, fs::File, io::Read, path::Path};

use encoding_rs_io::DecodeReaderBytesBuilder;
use regex::{Captures, Regex, RegexSet};
use serde::{Deserialize, Serialize};

use super::{
    error::{ParserAppError, ParserAppResult},
    player_info::LogfilePlayerInfo,
};

const MATCH_BLOCK_PATTERNS: [&str; 10] = [
    r"Match Started - \[\d+:(.+) /steam/(\d+)\], slot =\D+(\d)",
    r"Beginning mission (.+) \((\d) Humans, (\d) Computers\)",
    r"GAME -- Frame",
    r"SimID:(\d+), raceID:(\d+), teamID:(\d+), uid:\d+:(\d+), result:\d{1}:(.+)",
    r"SimID:(\d+), raceID:(\d+), teamID:(\d+), uid:\[\d+:(.+)\]",
    r"ReportSimStats - storing simulation results for match \d:(\d+)",
    r"Ending mission - '(\D+)'",
    r"Game Over at frame (\d+)",
    r"pid 0:(\d+), /steam/(\d+)",
    r"Found profile: /steam/(\d+)",
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
        r"Found profile",
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

impl LogfileGameInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block_complete(&self) -> bool {
        self.complete
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct LogfileGameList {
    logfile_content: Vec<String>,
    pub games: Vec<LogfileGameInfo>,
    pub steam_id: usize,
}

#[derive(Debug, Default)]
pub struct SteamIdMap {
    relic_id: usize,
    slot: usize,
    uid: String, // Game internal user id per player that is assigned when the match starts. Will be used to identify dropped players.
}

#[derive(Debug)]
struct MatchGroup<'a> {
    index: usize,
    captures: Captures<'a>,
}

impl LogfileGameList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_logfile(&mut self, logfilepath: &Path) -> ParserAppResult<()> {
        tracing::debug!("Reading logfile");
        if !logfilepath.exists() {
            tracing::error!("Could not find logfile!");
            return Err(ParserAppError::LogfileNotFoundError);
        }

        // Rust can not directly read from this file since it is not UTF-8 encoded
        let logfile = File::open(logfilepath)?;
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
        tracing::debug!("Done reading logfile");
        Ok(())
    }

    pub fn parse(&mut self) -> ParserAppResult<()> {
        tracing::debug!("Parsing logfile");
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
                tracing::error!("Found more than 1 game to parse");
                return Err(ParserAppError::LogfileParseError(
                    "Found more than 1 game to parse".into(),
                ));
            }

            let match_group = &match_captures[0];

            match match_group.index {
                0 => {
                    let Some(uid) = match_group.captures.get(1) else {
                        tracing::error!("Could not parse user id from match header block");
                        return Err(ParserAppError::LogfileParseError("Could not parse user id from match header block".into()));
                    };

                    let Some(steam_id) = match_group.captures.get(2) else {
                        tracing::error!("Could not parse steam id from match header block");
                        return Err(ParserAppError::LogfileParseError("Could not parse steam id from match header block".into()));
                    };

                    let Some(slot) = match_group.captures.get(3) else {
                        tracing::error!("Could not parse slot number from match header block");
                        return Err(ParserAppError::LogfileParseError("Could not parse slot number from match header block".into()));
                    };

                    let uid = uid.as_str().into();
                    let steam_id = steam_id.as_str().parse::<usize>().unwrap_or_default();
                    let slot = slot.as_str().parse::<usize>().unwrap_or_default();

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
                        tracing::error!("Could not parse map from logfile");
                        return Err(ParserAppError::LogfileParseError("Could not parse map from logfile".into()));
                    };

                    let len = self.games.len() - 1;
                    self.games[len].map = map.as_str().to_string();
                }
                2 => (),
                3 => {
                    if self.games.last_mut().is_some() {
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
                    if self.games.last_mut().is_some() {
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
                            tracing::error!("Could not extract match relic id from logfile");
                            return Err(ParserAppError::LogfileParseError("Could not extract match relic id from logfile".into()));
                        };

                        let Ok(match_relic_id) = match_relic_id.as_str().parse::<usize>() else {
                            tracing::error!("Could not parse match relic id in logfile");
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
                                    tracing::debug!("Game ended regularly");
                                    last_game.aborted = false;
                                    last_game.complete = true;
                                }
                                "Abort" => {
                                    tracing::debug!("Game was aborted");
                                    last_game.aborted = true;
                                    last_game.complete = true;
                                }
                                // Unknown status - defaulting to a cancelled and complete game
                                status => {
                                    tracing::error!("Unknown game status found: {:?}. Setting game status to aborted. Completing", status);
                                    last_game.aborted = true;
                                    last_game.complete = true;
                                }
                            },
                            // Could not find the game ending status information - defaulting to a
                            // cancelled and complete game
                            None => {
                                tracing::error!("No game status found. Setting game status to aborted. Completing");
                                last_game.aborted = true;
                                last_game.complete = true;
                            }
                        }
                    }
                }
                7 => {
                    if let Some(last_game) = self.games.last_mut() {
                        let Some(frames) = match_group.captures.get(1) else {
                            tracing::error!("Could not extract number of frames from logfile");
                            return Err(ParserAppError::LogfileParseError("Could not extract number of frames from logfile".into()));
                        };

                        let Ok(frames) = frames.as_str().parse::<usize>() else {
                            tracing::error!("Could not parse number of frames from logfile");
                            return Err(ParserAppError::LogfileParseError("Could not parse number of frames from logfile".into()));
                        };

                        tracing::debug!("Found {:?} frames in logfile", frames);

                        last_game.frames = frames;
                    }
                }
                8 => {
                    let Some(relic_id) = match_group.captures.get(1) else {
                        tracing::error!("Could not parse relic id from logfile");
                        return Err(ParserAppError::LogfileParseError("Could not parse relic id from logfile".into()));
                    };

                    let Some(steam_id) = match_group.captures.get(2) else {
                        tracing::error!("Could not parse steam id from logfile");
                        return Err(ParserAppError::LogfileParseError("Could not parse steam id from logfile".into()));
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
                9 => {
                    let Some(steam_id) = match_group.captures.get(1) else {
                        tracing::error!("Could not find player profile steam id in logfile");
                        return Err(ParserAppError::LogfileParseError("could not find player profile steam id in logfile".into()));
                    };

                    tracing::debug!("Found players steam profile: /steam/{:?}", steam_id);

                    if let Ok(steam_id) = steam_id.as_str().parse::<usize>() {
                        self.steam_id = steam_id;
                    } else {
                        tracing::error!("Could not read player profile steam id from logfile");
                        return Err(ParserAppError::LogfileParseError("could not read player profile steam id from logfile".into()));
                    }
                }
                capture_group => {
                    tracing::error!("RegEx error while parsing logfile: {:?}", capture_group);
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

fn contains_desired_content(string: &str) -> bool {
    LOGFILE_FILTER_REGEXP
        .matches(string)
        .into_iter()
        .next()
        .is_some()
}

#[cfg(test)]
mod tests {

    use crate::core::{logfile::LogfileGameList, player_info::LogfilePlayerStatus};

    use super::*;

    #[test]
    fn read_match_from_logfile() {
        let logfilepath = Path::new("warnings.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();

        assert_eq!(game_list.logfile_content.len(), 241);
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

    #[test]
    fn player_steam_id_is_correctly_read_from_logfile() {
        let logfilepath = Path::new("warnings.txt");
        let mut game_list = LogfileGameList::new();
        game_list.read_logfile(logfilepath).unwrap();
        game_list.parse().unwrap();

        assert_eq!(game_list.steam_id, 76561198099396483);
    }
}
