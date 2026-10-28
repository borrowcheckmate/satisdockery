use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::Package;
use crate::{Error, Result};

/// Endpoint to the ficsit.app REST api (<https://ficsit.app/api-docs>)
const REST_API: &str = "https://api.ficsit.app/";

#[derive(Deserialize)]
struct PluginDescriptor {
    #[serde(default, rename = "GameFeature")]
    game_feature: bool,
}

/// Downloads every package and extracts it into the installation's mods directory.
///
/// # Errors
/// Fails when a download fails or an archive is not a valid mod.
pub async fn install(install_dir: &Path, packages: &[Package]) -> Result<()> {
    let client = reqwest::Client::new();
    for package in packages {
        log::info!("Installing {} {}...", package.name, package.version);
        let url = format!("{REST_API}{}", package.link.trim_start_matches('/'));
        let archive = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        extract(install_dir, &package.name, &archive)?;
        log::info!("Installed {} {}", package.name, package.version);
    }
    Ok(())
}

fn extract(install_dir: &Path, name: &str, archive: &[u8]) -> Result<()> {
    ensure_valid_directory_name(name)?;
    let mut archive = zip::ZipArchive::new(Cursor::new(archive))?;
    let descriptor: PluginDescriptor =
        serde_json::from_reader(archive.by_name(&format!("{name}.uplugin"))?)?;
    archive.extract(plugin_directory(install_dir, name, &descriptor))?;
    Ok(())
}

/// Mod references become directory names, so references from API metadata must not escape.
fn ensure_valid_directory_name(name: &str) -> Result<()> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        return Err(Error::InvalidModReference(name.to_owned()));
    }
    Ok(())
}

fn plugin_directory(install_dir: &Path, name: &str, descriptor: &PluginDescriptor) -> PathBuf {
    let mut mods = install_dir.join("FactoryGame/Mods");
    if descriptor.game_feature {
        mods.push("GameFeatures");
    }
    mods.join(name)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use zip::write::SimpleFileOptions;

    use super::*;

    fn mod_archive(name: &str, descriptor: &str) -> Vec<u8> {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        archive
            .start_file(format!("{name}.uplugin"), options)
            .unwrap();
        archive.write_all(descriptor.as_bytes()).unwrap();
        archive
            .start_file("Binaries/Linux/module.so", options)
            .unwrap();
        archive.write_all(b"module").unwrap();
        archive.finish().unwrap().into_inner()
    }

    fn install_dir(test: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("satisdockery-{test}-{}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn installs_archives_in_their_plugin_directories() {
        let dir = install_dir("plugin-directories");
        for (name, descriptor, directory) in [
            ("SML", "{}", "SML"),
            ("NormalMod", r#"{"GameFeature":false}"#, "NormalMod"),
            (
                "FeatureMod",
                r#"{"GameFeature":true}"#,
                "GameFeatures/FeatureMod",
            ),
        ] {
            extract(&dir, name, &mod_archive(name, descriptor)).unwrap();

            let installed = dir.join("FactoryGame/Mods").join(directory);
            assert_eq!(
                fs::read(installed.join(format!("{name}.uplugin"))).unwrap(),
                descriptor.as_bytes()
            );
            assert_eq!(
                fs::read(installed.join("Binaries/Linux/module.so")).unwrap(),
                b"module"
            );
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn leaves_saves_and_other_mods_untouched() {
        let dir = install_dir("untouched");
        fs::create_dir_all(dir.join("FactoryGame/Mods/Existing")).unwrap();
        fs::write(
            dir.join("FactoryGame/Mods/Existing/Existing.uplugin"),
            b"{}",
        )
        .unwrap();
        fs::create_dir_all(dir.join("FactoryGame/Saved")).unwrap();
        fs::write(dir.join("FactoryGame/Saved/world.sav"), b"save").unwrap();

        extract(&dir, "NewMod", &mod_archive("NewMod", "{}")).unwrap();

        assert_eq!(
            fs::read(dir.join("FactoryGame/Saved/world.sav")).unwrap(),
            b"save"
        );
        assert!(
            dir.join("FactoryGame/Mods/Existing/Existing.uplugin")
                .is_file()
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_invalid_archives_and_names_without_installing_anything() {
        let dir = install_dir("rejects");
        let archive = mod_archive("Mod", "{}");

        assert!(extract(&dir, "MissingDescriptor", &archive).is_err());
        assert!(extract(&dir, "BrokenMod", b"not a zip").is_err());
        for invalid in ["", ".", "..", "../escape", "nested/mod", "nested\\mod"] {
            assert!(extract(&dir, invalid, &archive).is_err());
        }

        assert!(!dir.join("FactoryGame/Mods").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
