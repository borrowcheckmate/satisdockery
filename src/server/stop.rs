use std::io;

use tokio::process::Child;

#[cfg(unix)]
pub(super) struct Stop {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(not(unix))]
pub(super) struct Stop;

impl Stop {
    #[cfg(unix)]
    pub(super) fn new() -> io::Result<Self> {
        use tokio::signal::unix::{SignalKind, signal};

        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
        })
    }

    #[cfg(not(unix))]
    pub(super) const fn new() -> io::Result<Self> {
        Ok(Self)
    }

    #[cfg(unix)]
    pub(super) async fn wait(&mut self) {
        tokio::select! {
            _ = self.interrupt.recv() => {}
            _ = self.terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    pub(super) async fn wait(&mut self) {
        if let Err(error) = tokio::signal::ctrl_c().await {
            log::warn!("Could not listen for an interrupt: {error}");
            std::future::pending::<()>().await;
        }
    }
}

#[cfg(unix)]
pub(super) async fn interrupt(child: &mut Child) -> io::Result<std::process::ExitStatus> {
    use nix::{
        errno::Errno,
        sys::signal::{Signal, kill},
        unistd::Pid,
    };

    if let Some(id) = child.id() {
        let pid = i32::try_from(id).map_err(io::Error::other)?;
        match kill(Pid::from_raw(pid), Signal::SIGINT) {
            Ok(()) | Err(Errno::ESRCH) => {}
            Err(error) => return Err(io::Error::from_raw_os_error(error as i32)),
        }
    }
    child.wait().await
}

#[cfg(not(unix))]
pub(super) async fn interrupt(child: &mut Child) -> io::Result<std::process::ExitStatus> {
    child.kill().await?;
    child.wait().await
}

pub(super) fn interrupted(status: std::process::ExitStatus) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        status.signal() == Some(nix::libc::SIGINT)
    }
    #[cfg(not(unix))]
    {
        let _status = status;
        true
    }
}
