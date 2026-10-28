pub mod cli;
pub mod depot;
pub mod error;
pub mod game;

pub use error::{Error, Result};

use cli::Args;

/// Prepares the installation (game, mods) and then starts the server.
///
/// # Errors
/// Fails when any preparation step or the server process itself fails.
pub async fn run(args: &Args) -> Result<()> {
    if !args.skip_update {
        depot::download_dedicated_server(&args.path).await?;
    }
    let game_version = game::GameVersion::read(&args.path).await?;
    log::info!("Current game version: {}", game_version.game_version);
    log::info!(
        "Current mod build requirement: {}",
        game_version.mod_build_requirement()
    );
    Ok(())
}
