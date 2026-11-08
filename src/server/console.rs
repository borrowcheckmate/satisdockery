use std::{
    future,
    io::{self, BufRead},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use tokio::sync::mpsc;

use super::api::Api;

pub(super) fn input() -> io::Result<mpsc::Receiver<String>> {
    let (send, receive) = mpsc::channel(1);
    // Terminal reads may block beyond server exit. An ordinary thread does not hold up Tokio.
    drop(
        thread::Builder::new()
            .name("server-console".into())
            .spawn(move || read_lines(&send))?,
    );
    Ok(receive)
}

fn read_lines(send: &mpsc::Sender<String>) {
    for line in io::stdin().lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                log::warn!("Could not read the server console: {error}");
                break;
            }
        };
        if send.blocking_send(line).is_err() {
            break;
        }
    }
}

pub(super) async fn forward(input: Option<(&Api, mpsc::Receiver<String>)>, running: &AtomicBool) {
    let Some((api, mut lines)) = input else {
        return future::pending().await;
    };
    while let Some(line) = lines.recv().await {
        let command = line.trim();
        if command.is_empty() {
            continue;
        }
        if running.load(Ordering::Relaxed) {
            show_result(api.run_command(command).await);
        } else {
            log::warn!("The game is stopped, so the console command was discarded");
        }
    }
    future::pending::<()>().await;
}

fn show_result(result: crate::Result<(String, bool)>) {
    match result {
        Ok((output, success)) => {
            for line in output.lines() {
                log::info!("Console: {line}");
            }
            if !success {
                log::warn!("The game reported a console command failure");
            }
        }
        Err(error) => log::warn!("Could not send console command: {error}"),
    }
}
