use std::path::PathBuf;

use clap::Parser;

use crate::server::ServerSettings;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Path to Satisfactory installation
    #[arg(long, env = "SATISDOCKERY_PATH")]
    pub path: PathBuf,
    /// Skip running the updater
    #[arg(long, env = "SATISDOCKERY_SKIP_UPDATE")]
    pub skip_update: bool,
    /// Only perform pre-run and don't start server
    #[arg(long, env = "SATISDOCKERY_SKIP_RUN")]
    pub skip_run: bool,
    /// Use the latest experimental Satisfactory version
    #[arg(long, env = "SATISDOCKERY_EXPERIMENTAL")]
    pub experimental: bool,
    /// Mods to install, for example: RefinedPower:>=4.0.44,SecondMod:>=1.0.10
    #[arg(long, env = "SATISDOCKERY_MODS", value_delimiter = ',')]
    pub mods: Vec<String>,
    #[command(flatten)]
    pub server: ServerSettings,
}
