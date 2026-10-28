use std::{error::Error, fs, path::Path, process::Command};

type TestResult = Result<(), Box<dyn Error>>;

fn write_version_file(install_dir: &Path, changelist: u64) -> TestResult {
    let binaries = install_dir.join("Engine/Binaries/Linux");
    fs::create_dir_all(&binaries)?;
    fs::write(
        binaries.join("FactoryServer-Linux-Shipping.version"),
        serde_json::json!({
            "MajorVersion": 5,
            "MinorVersion": 3,
            "PatchVersion": 2,
            "Changelist": changelist,
            "CompatibleChangelist": 400_000,
            "IsLicenseeVersion": 1,
            "IsPromotedBuild": 1,
            "BranchName": "test",
            "BuildId": changelist.to_string(),
            "GameVersion": "1.1.0.0"
        })
        .to_string(),
    )?;
    Ok(())
}

fn prepare(install_dir: &Path) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_satisdockery"))
        .arg("--path")
        .arg(install_dir)
        .args(["--skip-update", "--skip-run"])
        .output()
}

#[test]
fn startup_uses_changelist_for_mod_compatibility() -> TestResult {
    let install_dir =
        std::env::temp_dir().join(format!("satisdockery-startup-{}", std::process::id()));
    write_version_file(&install_dir, 424_242)?;

    let output = prepare(&install_dir)?;
    fs::remove_dir_all(&install_dir)?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Current mod build requirement: 424242.0.0"),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn startup_fails_when_the_installation_has_no_version_file() -> TestResult {
    let install_dir =
        std::env::temp_dir().join(format!("satisdockery-empty-{}", std::process::id()));
    fs::create_dir_all(&install_dir)?;

    let output = prepare(&install_dir)?;
    fs::remove_dir_all(&install_dir)?;

    assert!(!output.status.success());
    Ok(())
}
