use std::process::Command;

use crate::{Error, Result};

/// Runs `command` with inherited stdio and waits for it to finish.
///
/// # Errors
/// Fails when the process cannot be started or exits unsuccessfully.
pub fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed {
            program: command.get_program().to_string_lossy().into_owned(),
            status,
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn succeeds_when_the_process_succeeds() {
        assert!(run(&mut Command::new("true")).is_ok());
    }

    #[test]
    fn reports_the_program_when_the_process_fails() {
        let result = run(&mut Command::new("false"));

        assert!(matches!(result, Err(Error::CommandFailed { program, .. }) if program == "false"));
    }

    #[test]
    fn fails_when_the_process_cannot_be_started() {
        assert!(matches!(
            run(&mut Command::new("/nonexistent/satisdockery")),
            Err(Error::Io(_))
        ));
    }
}
