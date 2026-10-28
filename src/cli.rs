use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, arg_required_else_help = true)]
pub struct Args {
    /// Path to Satisfactory installation
    #[arg(long)]
    pub path: PathBuf,
    /// Skip running the updater
    #[arg(long)]
    pub skip_update: bool,
}
