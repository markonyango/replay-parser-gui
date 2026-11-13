use chrono::prelude::*;
use parser_lib::{actions::Action, message::Message};
use serde::Serialize;

use super::{
    game::{map_slug, ExtendedGameInformation},
    player_info::{ExtendedPlayerInformation, LogfilePlayerStatus},
};

/// Sent when no player won. esl-report.php stores `winner + 1`, and the stored 0
/// means "winner cannot be determined", so ratings are left untouched. It also
/// matches no team, so no player gets a win.
const UNKNOWN_WINNER: i32 = -1;

#[derive(Debug, Serialize)]
pub struct ReplayReportDto {
    aborted: bool,
    actions: Vec<Action>,
    /// Every decoded command, sent as parsed, `command` and `command_error` included.
    commands: Vec<Action>,
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
    winner: i32,
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
            .map_or(UNKNOWN_WINNER, |player| player.team as i32);

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
                    command: None,
                    command_error: None,
                })
                .collect(),
            commands: replay.commands.clone(),
            dev: replay.dev.unwrap_or_default(),
            id: replay.id.to_string(),
            map: map_slug(&replay.map),
            reporter: ReplayReportReporterDto {
                date: chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                version: "1.0.9".into(),
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
            winner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with(statuses: &[(u32, LogfilePlayerStatus)]) -> ExtendedGameInformation {
        ExtendedGameInformation {
            players: statuses
                .iter()
                .map(|(team, status)| ExtendedPlayerInformation {
                    team: *team,
                    status: status.clone(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn reports_team_of_winning_player() {
        let game = game_with(&[
            (0, LogfilePlayerStatus::Killed),
            (1, LogfilePlayerStatus::Won),
        ]);

        assert_eq!(ReplayReportDto::from(&game).winner, 1);
    }

    #[test]
    fn reports_unknown_winner_when_nobody_won() {
        let game = game_with(&[
            (0, LogfilePlayerStatus::Unknown),
            (1, LogfilePlayerStatus::Playing),
        ]);

        assert_eq!(ReplayReportDto::from(&game).winner, UNKNOWN_WINNER);
    }

    #[test]
    fn reports_version_1_0_9() {
        let json = serde_json::to_value(ReplayReportDto::from(&game_with(&[]))).unwrap();

        assert_eq!(json["reporter"]["version"], "1.0.9");
    }

    #[test]
    fn upload_carries_every_decoded_command() {
        let replay = parser_lib::parse_raw("3v3.rec".to_string()).unwrap();
        let game = ExtendedGameInformation {
            commands: replay.commands.clone(),
            ..Default::default()
        };

        let json = serde_json::to_value(ReplayReportDto::from(&game)).unwrap();
        let commands = json["commands"].as_array().unwrap();

        assert!(!commands.is_empty(), "fixture has commands");
        assert_eq!(commands.len(), replay.commands.len());
        for command in commands {
            assert!(command["tick"].is_u64(), "{command}");
            assert!(command["relic_id"].is_u64(), "{command}");
            assert!(command["data"]
                .as_array()
                .unwrap()
                .iter()
                .all(|b| b.as_u64().is_some_and(|b| b <= 255)));
            let error = &command["command_error"];
            assert!(error.is_null() || error.is_string(), "{command}");
            assert!(
                command["command"].is_null() || command["command"]["opcode"].is_u64(),
                "{command}"
            );
        }

        let (index, decoded) = replay
            .commands
            .iter()
            .enumerate()
            .find(|(_, c)| {
                c.command
                    .as_ref()
                    .is_some_and(|c| c.opcode.name().is_some())
            })
            .expect("fixture has a named decoded command");
        let sent = &commands[index]["command"];
        let decoded = decoded.command.as_ref().unwrap();
        assert_eq!(sent["opcode"], u64::from(decoded.opcode.value()));
        assert_eq!(sent["opcode_name"], decoded.opcode.name().unwrap());

        // Blueprints are only resolved when the game data is installed.
        if let Some((index, with_blueprint)) = replay
            .commands
            .iter()
            .enumerate()
            .find(|(_, c)| c.command.as_ref().is_some_and(|c| c.blueprint.is_some()))
        {
            let blueprint = with_blueprint.command.as_ref().unwrap().blueprint.as_ref();
            assert_eq!(
                commands[index]["command"]["blueprint"],
                serde_json::to_value(blueprint).unwrap()
            );
        }
    }
}
