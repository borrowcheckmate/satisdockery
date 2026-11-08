use std::{
    io,
    net::{Ipv4Addr, TcpListener},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use tokio::{
    process::{Child, Command as AsyncCommand},
    time,
};

use super::{ServerSettings, api::Api, console, idle, stop, wake};
use crate::{Result, command};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) async fn run(mut command: Command, settings: &ServerSettings) -> Result<()> {
    let api = configure_api(&mut command, settings).await?;
    let input = if settings.console {
        command.stdin(Stdio::null());
        api.as_ref()
            .map(|api| console::input().map(|input| (api, input)))
            .transpose()?
    } else {
        None
    };
    let idle = settings
        .idle_timeout
        .zip(api.as_ref())
        .map(|(timeout, api)| (api, Duration::from_secs(timeout.get())));
    let running = AtomicBool::new(false);
    tokio::select! {
        result = supervise(AsyncCommand::from(command), idle, settings.game_port, &running) => result?,
        () = console::forward(input, &running) => {}
    }
    log::info!("Server shutdown complete");
    Ok(())
}

async fn configure_api(command: &mut Command, settings: &ServerSettings) -> Result<Option<Api>> {
    if !settings.console && settings.idle_timeout.is_none() {
        return Ok(None);
    }
    if settings.game_port == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Server API access requires a fixed game port",
        )
        .into());
    }
    // Check that the local API port does not already belong to another process.
    drop(TcpListener::bind((
        Ipv4Addr::LOCALHOST,
        settings.game_port,
    ))?);
    let api = Api::new(settings).await?;
    if settings.trust_local_api && settings.idle_api_token_file.is_none() {
        command.arg("-ini:Engine:[SystemSettings]:FG.DedicatedServer.AllowInsecureLocalAccess=1");
        log::info!("Enabled Satisfactory admin API access for loopback clients");
    }
    Ok(Some(api))
}

async fn supervise(
    mut command: AsyncCommand,
    idle: Option<(&Api, Duration)>,
    port: u16,
    running: &AtomicBool,
) -> Result<()> {
    let mut stop = stop::Stop::new()?;
    if idle.is_some() {
        drop(wake::bind(port)?);
    }

    loop {
        let mut child = command.spawn()?;
        running.store(true, Ordering::Relaxed);
        let should_sleep = wait_child(&mut child, command.as_std(), idle, &mut stop).await?;
        running.store(false, Ordering::Relaxed);
        if !should_sleep {
            return Ok(());
        }

        let socket = wake::bind(port)?;
        log::info!("Game stopped while idle and waiting for a UDP status request");
        tokio::select! {
            biased;
            () = stop.wait() => return Ok(()),
            result = wake::wait(socket) => result?,
        }
        log::info!("UDP status request received, starting the game");
    }
}

async fn wait_child(
    child: &mut Child,
    command: &Command,
    idle: Option<(&Api, Duration)>,
    stop: &mut stop::Stop,
) -> Result<bool> {
    loop {
        tokio::select! {
            biased;
            () = stop.wait() => return stop_child(child, command).await,
            () = idle::monitor(idle) => {}
            status = child.wait() => {
                command::check_status(command, status?)?;
                return Ok(false);
            }
        }
        tokio::select! {
            biased;
            () = stop.wait() => return stop_child(child, command).await,
            result = time::timeout(SHUTDOWN_TIMEOUT, child.wait()) => {
                if let Ok(status) = result {
                    command::check_status(command, status?)?;
                    return Ok(true);
                }
            }
        }
        log::warn!("Game did not exit after the idle shutdown request");
    }
}

async fn stop_child(child: &mut Child, command: &Command) -> Result<bool> {
    let status = stop::interrupt(child).await?;
    if !stop::interrupted(status) {
        command::check_status(command, status)?;
    }
    Ok(false)
}
