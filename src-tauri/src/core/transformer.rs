use crate::core::player_info::ExtendedPlayerInformation;
use parser_lib::replay::ReplayInfo;

pub fn transform_player_information(
    logfile: &super::logfile::LogfileGameInfo,
    replay: &ReplayInfo,
) -> Vec<ExtendedPlayerInformation> {
    let players_with_extended_information = replay
        .players
        .iter()
        .map(|replayfile_player| {
            let _player = match replayfile_player {
                parser_lib::chunky::Chunk::Player(player) => Some(player),
                _ => None,
            };

            // The logfile lists players in a different order than the replay and may
            // contain extra entries (AI, dropped players), so join on the relic id.
            // AI and unresolved entries carry relic id 0 and must not match anyone.
            let logfile_player = _player
                .filter(|player| player.relic_id != 0)
                .and_then(|player| {
                    logfile
                        .players
                        .iter()
                        .find(|p| p.relic_id as u64 == player.relic_id)
                });

            match (logfile_player, _player) {
                (Some(logfile_player), Some(player)) => {
                    ExtendedPlayerInformation::from(logfile_player, &player)
                }
                // No matching logfile entry (e.g. playback or unparsed log): keep what the replay knows.
                (None, Some(player)) => ExtendedPlayerInformation::from_replay(player),
                _ => ExtendedPlayerInformation::default(),
            }
        })
        .collect::<Vec<_>>();

    players_with_extended_information
}

#[cfg(test)]
mod tests {
    use parser_lib::chunky::Chunk;

    use super::*;
    use crate::core::{logfile::LogfileGameInfo, player_info::LogfilePlayerInfo};

    fn replay_relic_ids(replay: &ReplayInfo) -> Vec<u64> {
        replay
            .players
            .iter()
            .filter_map(|chunk| match chunk {
                Chunk::Player(player) => Some(player.relic_id),
                _ => None,
            })
            .collect()
    }

    /// A logfile player whose logfile-only fields are derived from its relic id,
    /// so a wrong pairing is visible in the result.
    fn logfile_player(relic_id: usize) -> LogfilePlayerInfo {
        LogfilePlayerInfo {
            relic_id,
            steam_id: relic_id + 1,
            sim_id: relic_id + 2,
            race: relic_id % 7,
            ..Default::default()
        }
    }

    #[test]
    fn joins_players_by_relic_id_regardless_of_order() {
        let replay = parser_lib::parse_raw("3v3.rec".to_string()).unwrap();
        let relic_ids = replay_relic_ids(&replay);

        // Reverse the replay order and put an AI entry (no relic id) in the middle,
        // the way warnings.txt lists them.
        let mut players = relic_ids
            .iter()
            .rev()
            .map(|id| logfile_player(*id as usize))
            .collect::<Vec<_>>();
        players.insert(2, LogfilePlayerInfo::default());

        let logfile = LogfileGameInfo {
            players,
            ..Default::default()
        };

        let result = transform_player_information(&logfile, &replay);

        assert_eq!(result.len(), relic_ids.len());
        for player in &result {
            let expected = logfile_player(player.relic_id as usize);
            assert_eq!(player.steam_id, expected.steam_id, "{}", player.name);
            assert_eq!(player.sim_id, expected.sim_id, "{}", player.name);
            assert_eq!(player.race, expected.race, "{}", player.name);
        }
    }

    #[test]
    fn keeps_replay_data_for_players_missing_from_logfile() {
        let replay = parser_lib::parse_raw("3v3.rec".to_string()).unwrap();
        let relic_ids = replay_relic_ids(&replay);

        let logfile = LogfileGameInfo {
            players: vec![logfile_player(relic_ids[1] as usize)],
            ..Default::default()
        };

        let result = transform_player_information(&logfile, &replay);

        assert_eq!(result[0].relic_id, relic_ids[0]);
        assert_eq!(result[0].steam_id, 0);
        assert_eq!(result[0].name, "Venniie");
        assert_eq!(result[1].steam_id, relic_ids[1] as usize + 1);
    }
}
