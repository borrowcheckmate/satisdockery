use super::ServerSettings;

/// Builds the command line arguments that configure the Unreal server from `settings`.
pub fn unreal_arguments(settings: &ServerSettings) -> Vec<String> {
    let engine = |section: &str, key: &str, value: String| ini("Engine", section, key, &value);
    let game = |section: &str, key: &str, value: String| ini("Game", section, key, &value);
    let net_driver = "/Script/OnlineSubsystemUtils.IpNetDriver";
    let game_session = "/Script/Engine.GameSession";
    let timeout = settings.max_timeout.to_string();
    let tickrate = settings.max_tickrate.to_string();
    let players = settings.players.to_string();

    let mut arguments = vec![
        format!("-Port {}", settings.game_port),
        format!("-ReliablePort {}", settings.messaging_port),
        format!("-ExternalReliablePort {}", settings.messaging_port),
        engine(
            "/Script/FactoryGame.FGSaveSession",
            "mNumRotatingAutosaves",
            settings.max_autosaves.to_string(),
        ),
        engine(
            "/Script/Engine.GarbageCollectionSettings",
            "gc.MaxObjectsInEditor",
            settings.max_objects.to_string(),
        ),
        engine(net_driver, "LanServerMaxTickRate", tickrate.clone()),
        engine(net_driver, "NetServerMaxTickRate", tickrate),
        engine(net_driver, "ConnectionTimeout", timeout.clone()),
        engine(net_driver, "InitialConnectTimeout", timeout.clone()),
        game(game_session, "ConnectionTimeout", timeout.clone()),
        game(game_session, "InitialConnectTimeout", timeout),
        engine(
            "ConsoleVariables",
            "wp.Runtime.EnableServerStreaming",
            settings.enable_serverstreaming.to_string(),
        ),
        game(game_session, "MaxPlayers", players.clone()),
        ini("GameUserSettings", game_session, "MaxPlayers", &players),
    ];
    if settings.disable_seasonalevents {
        arguments.push("-DisableSeasonalEvents".to_owned());
    }
    arguments.extend(settings.additional_dedicatedserver_args.iter().cloned());
    arguments
}

fn ini(file: &str, section: &str, key: &str, value: &str) -> String {
    format!("-ini:{file}:[{section}]:{key}={value}")
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::cli::Args;

    fn settings(arguments: &[&str]) -> ServerSettings {
        let mut command = vec!["satisdockery", "--path", "/game"];
        command.extend_from_slice(arguments);
        Args::parse_from(command).server
    }

    #[test]
    fn defaults_expose_the_standard_ports() {
        let arguments = unreal_arguments(&settings(&[]));

        assert_eq!(arguments[0], "-Port 7777");
        assert_eq!(arguments[1], "-ReliablePort 8888");
        assert_eq!(arguments[2], "-ExternalReliablePort 8888");
    }

    #[test]
    fn applies_the_player_limit_to_both_config_files() {
        let arguments = unreal_arguments(&settings(&["--players", "9"]));

        assert!(
            arguments.contains(&"-ini:Game:[/Script/Engine.GameSession]:MaxPlayers=9".to_owned())
        );
        assert!(arguments.contains(
            &"-ini:GameUserSettings:[/Script/Engine.GameSession]:MaxPlayers=9".to_owned()
        ));
    }

    #[test]
    fn appends_switches_and_additional_arguments_last() {
        let arguments = unreal_arguments(&settings(&[
            "--disable-seasonalevents",
            "--additional-dedicatedserver-args",
            "-One,-Two",
        ]));

        assert_eq!(
            arguments[arguments.len() - 3..],
            ["-DisableSeasonalEvents", "-One", "-Two"]
        );
    }
}
