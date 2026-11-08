mod api;
mod wake;

use std::{
    io,
    net::{Ipv4Addr, TcpListener},
    num::NonZeroU64,
    process::Command,
    time::Duration,
};

use sha2::{Digest, Sha256};
use tokio::{
    process::Command as AsyncCommand,
    time::{self, Instant, MissedTickBehavior},
};

use super::ServerSettings;
use crate::{Error, Result};

use api::{Api, GameState};

const POLL_PERIOD: Duration = Duration::from_secs(15);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) async fn run(
    command: Command,
    settings: &ServerSettings,
    idle_timeout: NonZeroU64,
) -> Result<()> {
    if settings.game_port == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Idle shutdown requires a fixed game port",
        )
        .into());
    }
    let api = Api::new(settings).await?;
    let idle_timeout = Duration::from_secs(idle_timeout.get());

    // Check that the configured port does not already belong to another server.
    drop(TcpListener::bind((
        Ipv4Addr::LOCALHOST,
        settings.game_port,
    ))?);
    drop(wake::bind(settings.game_port)?);

    let mut command = AsyncCommand::from(command);
    while run_child(&mut command, &api, idle_timeout).await? {
        let socket = wake::bind(settings.game_port)?;
        log::info!("Game is stopped; waiting for a UDP status request");
        wake::wait(socket).await?;
        log::info!("UDP status request received; starting the game");
    }
    log::info!("Server shutdown completed");
    Ok(())
}

async fn run_child(command: &mut AsyncCommand, api: &Api, idle_timeout: Duration) -> Result<bool> {
    let mut child = command.spawn()?;
    let (status, idle_shutdown) = loop {
        tokio::select! {
            biased;
            () = monitor(api, idle_timeout) => {}
            status = child.wait() => break (status?, false),
        }
        match time::timeout(SHUTDOWN_TIMEOUT, child.wait()).await {
            Ok(status) => break (status?, true),
            Err(_) => log::warn!("Game did not exit after the idle shutdown request"),
        }
    };
    if status.success() {
        Ok(idle_shutdown)
    } else {
        Err(Error::CommandFailed {
            program: command
                .as_std()
                .get_program()
                .to_string_lossy()
                .into_owned(),
            status,
        })
    }
}

async fn monitor(api: &Api, idle_timeout: Duration) {
    let mut interval = time::interval(POLL_PERIOD.min(idle_timeout));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut idle = Idle::default();
    loop {
        interval.tick().await;
        let state = match api.state().await {
            Ok(state) => state,
            Err(error) => {
                log::debug!("Could not check whether the game is idle: {error}");
                idle = Idle::default();
                continue;
            }
        };
        let Some(session) = idle.observe(&state, Instant::now(), idle_timeout) else {
            continue;
        };
        match save_and_stop(api, session).await {
            Ok(true) => return,
            Ok(false) => {}
            Err(error) => log::warn!("Idle shutdown canceled: {error}"),
        }
        idle = Idle::default();
    }
}

async fn save_and_stop(api: &Api, session: &str) -> Result<bool> {
    let save_name = format!("SatisDockery_Idle_{}", hex::encode(Sha256::digest(session)));
    api.save(&save_name).await?;
    // A player may have joined, or the active session may have changed during the save.
    if api.state().await?.idle_session() != Some(session) {
        return Ok(false);
    }
    api.shutdown().await?;
    Ok(true)
}

impl GameState {
    fn idle_session(&self) -> Option<&str> {
        (self.num_connected_players == 0
            && self.is_game_running
            && !self.active_session_name.is_empty()
            && self.active_session_name == self.auto_load_session_name)
            .then_some(self.active_session_name.as_str())
    }
}

#[derive(Default)]
struct Idle {
    since: Option<(String, Instant)>,
}

impl Idle {
    fn observe<'a>(
        &mut self,
        state: &'a GameState,
        now: Instant,
        timeout: Duration,
    ) -> Option<&'a str> {
        let Some(session) = state.idle_session() else {
            self.since = None;
            return None;
        };
        if let Some((previous_session, since)) = &self.since
            && previous_session == session
        {
            return (now.duration_since(*since) >= timeout).then_some(session);
        }
        self.since = Some((session.to_owned(), now));
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_a_full_idle_period_in_the_same_autoloading_session() {
        let mut state = GameState {
            num_connected_players: 0,
            is_game_running: true,
            active_session_name: "factory".into(),
            auto_load_session_name: "factory".into(),
        };
        let timeout = Duration::from_secs(10);
        let start = Instant::now();
        let mut idle = Idle::default();
        assert_eq!(idle.observe(&state, start, timeout), None);
        assert_eq!(
            idle.observe(&state, start + timeout, timeout),
            Some("factory")
        );

        state.num_connected_players = 1;
        assert_eq!(idle.observe(&state, start + timeout, timeout), None);
        state.num_connected_players = 0;
        assert_eq!(idle.observe(&state, start + timeout, timeout), None);

        state.active_session_name = "other factory".into();
        assert_eq!(idle.observe(&state, start + timeout * 2, timeout), None);
        state
            .auto_load_session_name
            .clone_from(&state.active_session_name);
        assert_eq!(idle.observe(&state, start + timeout * 2, timeout), None);
        assert_eq!(
            idle.observe(&state, start + timeout * 3, timeout),
            Some("other factory")
        );
    }
}
