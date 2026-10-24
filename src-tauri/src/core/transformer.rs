use crate::core::player_info::ExtendedPlayerInformation;
use parser_lib::replay::ReplayInfo;

pub fn transform_player_information(
    logfile: &super::logfile::LogfileGameInfo,
    replay: &ReplayInfo,
) -> Vec<ExtendedPlayerInformation> {
    let players_with_extended_information = replay
        .players
        .iter()
        .enumerate()
        .map(|(index, replayfile_player)| {
            let logfile_player = logfile.players.get(index);

            let _player = match replayfile_player {
                parser_lib::chunky::Chunk::Player(player) => Some(player),
                _ => None,
            };

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
