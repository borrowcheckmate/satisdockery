mod checksum;
mod release;

use std::{
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{Result, command};

/// Satisfactory Steam app id
const APP_ID: &str = "1690800";
const EXECUTABLE: &str = "DepotDownloader";

/// Installs (or updates) the dedicated server in `install_dir` through `DepotDownloader`.
///
/// # Errors
/// Fails when `DepotDownloader` cannot be installed and verified, or when it exits unsuccessfully.
pub async fn download_dedicated_server(install_dir: &Path) -> Result<()> {
    download_branch(install_dir, false).await
}

pub(crate) async fn download_branch(install_dir: &Path, experimental: bool) -> Result<()> {
    let branch = steam_branch(experimental);
    log::info!("Updating Satisfactory dedicated server from the {branch} branch");
    let executable = ensure_depot_downloader(install_dir).await?;
    command::run(&mut download_command(&executable, install_dir, branch))?;
    log::info!("Satisfactory dedicated server update complete");
    Ok(())
}

const fn steam_branch(experimental: bool) -> &'static str {
    if experimental {
        "experimental"
    } else {
        "public"
    }
}

fn download_command(executable: &Path, install_dir: &Path, branch: &str) -> Command {
    let mut command = Command::new(executable);
    command.args(["-app", APP_ID, "-os", "linux", "-branch", branch, "-dir"]);
    command.arg(install_dir);
    command
}

async fn ensure_depot_downloader(install_dir: &Path) -> Result<PathBuf> {
    let build = release::current_build()?;
    let directory = install_dir.join("depot_downloader");
    let executable = directory.join(EXECUTABLE);
    if executable.is_file() && checksum::verify(&executable, build.sha256).is_ok() {
        log::debug!("Reusing verified DepotDownloader");
        return Ok(executable);
    }

    log::info!("Downloading DepotDownloader");
    let archive = reqwest::get(build.url).await?.bytes().await?;
    tokio::fs::create_dir_all(&directory).await?;
    zip::ZipArchive::new(Cursor::new(archive))?.extract(&directory)?;
    checksum::verify(&executable, build.sha256)?;
    log::info!("Installed DepotDownloader");
    Ok(executable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_each_branch_in_the_same_installation() {
        let install_dir = Path::new("game");
        for (experimental, branch) in [(false, "public"), (true, "experimental"), (false, "public")]
        {
            let command = download_command(
                Path::new(EXECUTABLE),
                install_dir,
                steam_branch(experimental),
            );
            assert_eq!(
                command.get_args().collect::<Vec<_>>(),
                [
                    "-app", APP_ID, "-os", "linux", "-branch", branch, "-dir", "game"
                ]
            );
        }
    }
}
