mod api;
mod arguments;
mod console;
mod idle;
mod permissions;
mod process;
mod settings;
mod stop;
mod wake;

use std::{path::Path, process::Command};

use crate::{Error, Result, command, game};

pub use settings::ServerSettings;

const EXECUTABLE: &str = "FactoryServer-Linux-Shipping";
const CRASH_HANDLER: &str = "crashpad_handler";

/// Starts the dedicated server in `install_dir` and blocks until it shuts down.
///
/// # Errors
/// Fails when the server binary cannot be prepared or exits unsuccessfully.
pub fn run(install_dir: &Path, settings: &ServerSettings) -> Result<()> {
    command::run(&mut server_command(install_dir, settings)?)?;
    log::info!("Server shutdown completed");
    Ok(())
}

pub(crate) async fn run_configured(install_dir: &Path, settings: &ServerSettings) -> Result<()> {
    process::run(server_command(install_dir, settings)?, settings).await
}

fn server_command(install_dir: &Path, settings: &ServerSettings) -> Result<Command> {
    let binaries = game::binaries_dir(install_dir);
    let executable = binaries.join(EXECUTABLE);
    permissions::make_executable(&executable)?;
    permissions::make_executable_if_present(&binaries.join(CRASH_HANDLER))?;

    let executable = executable
        .to_str()
        .ok_or_else(|| Error::NonUtf8Path(executable.clone()))?;
    let (program, arguments) = launch_command(executable, arguments::unreal_arguments(settings));
    let mut command = Command::new(program);
    command.args(arguments).env("UE_PROJECT_ROOT", install_dir);
    Ok(command)
}

/// The server is an `x86_64` binary, so other architectures run it through `box64`.
fn launch_command(executable: &str, server_arguments: Vec<String>) -> (String, Vec<String>) {
    let mut arguments = vec!["FactoryGame".to_owned()];
    arguments.extend(server_arguments);
    if std::env::consts::ARCH == "aarch64" {
        arguments.insert(0, executable.to_owned());
        ("box64".to_owned(), arguments)
    } else {
        (executable.to_owned(), arguments)
    }
}
