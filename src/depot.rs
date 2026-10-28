mod checksum;
mod release;

use std::{
    ffi::OsString,
    io::Cursor,
    path::{Path, PathBuf},
};

use duct::cmd;

use crate::Result;

/// Satisfactory Steam app id
const APP_ID: &str = "1690800";
const EXECUTABLE: &str = "DepotDownloader";

/// Installs (or updates) the dedicated server in `install_dir` through `DepotDownloader`.
///
/// # Errors
/// Fails when `DepotDownloader` cannot be installed and verified, or when it exits unsuccessfully.
pub async fn download_dedicated_server(install_dir: &Path) -> Result<()> {
    log::info!("Preparing Satisfactory Dedicated server...");
    let executable = ensure_depot_downloader(install_dir).await?;
    log::info!("Downloading Satisfactory Dedicated server...");
    run_depot_downloader(&executable, install_dir)?;
    log::info!("Satisfactory Dedicated server up-to date and ready to be launched.");
    Ok(())
}

fn run_depot_downloader(executable: &Path, install_dir: &Path) -> Result<()> {
    let arguments: Vec<OsString> = vec![
        "-app".into(),
        APP_ID.into(),
        "-os".into(),
        "linux".into(),
        "-dir".into(),
        install_dir.into(),
    ];
    cmd(executable, arguments).run()?;
    Ok(())
}

async fn ensure_depot_downloader(install_dir: &Path) -> Result<PathBuf> {
    log::info!("Checking DepotDownloader status");
    let build = release::current_build()?;
    let directory = install_dir.join("depot_downloader");
    let executable = directory.join(EXECUTABLE);
    if executable.is_file() && checksum::verify(&executable, build.sha256).is_ok() {
        log::info!("DepotDownloader already installed and hash matches.");
        return Ok(executable);
    }

    log::info!("Downloading DepotDownloader...");
    let archive = reqwest::get(build.url).await?.bytes().await?;
    tokio::fs::create_dir_all(&directory).await?;
    zip::ZipArchive::new(Cursor::new(archive))?.extract(&directory)?;
    checksum::verify(&executable, build.sha256)?;
    log::info!("DepotDownloader installed.");
    Ok(executable)
}
