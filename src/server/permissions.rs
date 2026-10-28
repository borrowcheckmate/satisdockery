use std::path::Path;

use crate::Result;

/// Depot downloads do not preserve the execute bit, so it is restored before launching.
#[cfg(unix)]
pub fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn make_executable_if_present(path: &Path) -> Result<()> {
    if path.is_file() {
        make_executable(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn adds_execute_bits_without_changing_other_permissions() {
        use std::{fs, os::unix::fs::PermissionsExt};

        let path =
            std::env::temp_dir().join(format!("satisdockery-executable-{}", std::process::id()));
        fs::write(&path, b"fixture").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        make_executable(&path).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        fs::remove_file(&path).unwrap();

        assert_eq!(mode, 0o751);
    }

    #[test]
    fn ignores_missing_files_when_optional() {
        assert!(make_executable_if_present(Path::new("/nonexistent/satisdockery")).is_ok());
    }
}
