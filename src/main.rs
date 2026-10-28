use std::process::ExitCode;

use clap::Parser;
use satisdockery::cli::Args;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();
    match satisdockery::run(&Args::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error}");
            ExitCode::FAILURE
        }
    }
}
