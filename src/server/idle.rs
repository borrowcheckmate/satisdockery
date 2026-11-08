use std::time::Duration;

use sha2::{Digest, Sha256};
use tokio::time::{self, Instant, MissedTickBehavior};

use super::api::{Api, GameState};
use crate::Result;

const POLL_PERIOD: Duration = Duration::from_secs(15);

pub(super) async fn monitor(config: Option<(&Api, Duration)>) {
    let Some((api, idle_timeout)) = config else {
        return std::future::pending().await;
    };
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
