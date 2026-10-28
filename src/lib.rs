pub mod cli;
pub mod command;
pub mod depot;
pub mod error;
pub mod game;
pub mod mods;
pub mod server;

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
    let build_requirement = game_version.mod_build_requirement();
    log::info!("Current mod build requirement: {build_requirement}");

    let packages = mods::resolve_packages(&args.mods, build_requirement).await?;
    log::info!("Resolved packages: {packages:?}");
    mods::install(&args.path, &packages).await?;
    log::info!("Preparation finished successfully!");

    if !args.skip_run {
        server::run(&args.path, &args.server)?;
    }
    Ok(())
}
