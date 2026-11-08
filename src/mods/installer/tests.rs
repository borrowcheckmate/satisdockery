use std::{fs, future, io::Write, path::PathBuf};

use zip::write::SimpleFileOptions;

use super::*;

fn mod_archive(name: &str, descriptor: &str) -> Vec<u8> {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
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
        install_fixture(&dir, name, &mod_archive(name, descriptor)).unwrap();

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

    install_fixture(&dir, "NewMod", &mod_archive("NewMod", "{}")).unwrap();

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

    assert!(install_fixture(&dir, "MissingDescriptor", &archive).is_err());
    assert!(install_fixture(&dir, "BrokenMod", b"not a zip").is_err());
    for invalid in ["", ".", "..", "../escape", "nested/mod", "nested\\mod"] {
        assert!(install_fixture(&dir, invalid, &archive).is_err());
    }

    assert!(
        Store::open(&dir, true)
            .unwrap()
            .unwrap()
            .find("BrokenMod")
            .unwrap()
            .is_none()
    );
    fs::remove_dir_all(dir).unwrap();
}

fn package(name: &str, version: u64) -> Package {
    Package {
        name: name.into(),
        version: Version::new(version, 0, 0),
        link: name.into(),
    }
}

fn install_fixture(dir: &Path, name: &str, archive: &[u8]) -> Result<()> {
    ensure_valid_directory_name(name)?;
    let store = Store::open(dir, true)?.unwrap();
    store.recover()?;
    install_package(&store, &package(name, 1), None, archive)
}

#[tokio::test]
async fn downloads_only_when_missing_or_changed_and_removes_owned_mods() {
    let dir = install_dir("reconcile");
    let normal = dir.join("FactoryGame/Mods/Mod");
    let feature = dir.join("FactoryGame/Mods/GameFeatures/Mod");
    let manual = dir.join("FactoryGame/Mods/Manual");
    fs::create_dir_all(&manual).unwrap();
    fs::write(manual.join("Manual.uplugin"), b"{}").unwrap();
    fs::create_dir_all(dir.join("FactoryGame/Saved")).unwrap();
    fs::write(dir.join("FactoryGame/Saved/world.sav"), b"save").unwrap();

    let mut downloads = 0;
    let mut fetch = |name: String| {
        let descriptors = ["{}", r#"{"GameFeature":true}"#];
        let archive = mod_archive(&name, descriptors[downloads]);
        downloads += 1;
        future::ready(Ok(archive))
    };
    reconcile(&dir, &[package("Mod", 1)], &mut fetch)
        .await
        .unwrap();
    reconcile(&dir, &[package("Mod", 1)], &mut fetch)
        .await
        .unwrap();
    reconcile(&dir, &[package("Mod", 2)], &mut fetch)
        .await
        .unwrap();
    assert_eq!(downloads, 2);
    assert!(!normal.exists());
    assert!(feature.join("Mod.uplugin").is_file());

    reconcile(&dir, &[], |_| future::ready(Ok(Vec::new())))
        .await
        .unwrap();
    assert!(!feature.exists());
    assert!(manual.join("Manual.uplugin").is_file());
    assert_eq!(
        fs::read(dir.join("FactoryGame/Saved/world.sav")).unwrap(),
        b"save"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn invalid_update_keeps_the_existing_version() {
    let dir = install_dir("invalid-update");
    install_fixture(&dir, "Mod", &mod_archive("Mod", "{}")).unwrap();
    assert!(
        reconcile(&dir, &[package("Mod", 2)], |_| future::ready(Ok(
            b"bad zip"
        )))
        .await
        .is_err()
    );
    let path = dir.join("FactoryGame/Mods/Mod");
    assert_eq!(
        Record::read(&path).unwrap().unwrap().version,
        Version::new(1, 0, 0)
    );
    assert!(path.join("Mod.uplugin").is_file());
    fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn matching_legacy_mod_is_reused_and_never_removed() {
    let dir = install_dir("legacy");
    let path = dir.join("FactoryGame/Mods/Legacy");
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("Legacy.uplugin"), br#"{"VersionName":"1.0.0"}"#).unwrap();
    let fetch = |_| {
        future::ready(Err::<Vec<u8>, _>(
            io::Error::other("unexpected download").into(),
        ))
    };
    reconcile(&dir, &[package("Legacy", 1)], fetch)
        .await
        .unwrap();
    assert!(
        reconcile(&dir, &[package("Legacy", 2)], fetch)
            .await
            .is_err()
    );
    reconcile(&dir, &[], fetch).await.unwrap();
    assert!(path.join("Legacy.uplugin").is_file());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn recovers_an_interrupted_replacement() {
    let dir = install_dir("recover-update");
    install_fixture(&dir, "Mod", &mod_archive("Mod", "{}")).unwrap();
    let path = dir.join("FactoryGame/Mods/Mod");
    let state = dir.join("FactoryGame/Mods/.satisdockery");
    fs::rename(&path, state.join("backup")).unwrap();
    fs::create_dir(state.join("staging")).unwrap();

    let store = Store::open(&dir, true).unwrap().unwrap();
    store.recover().unwrap();
    assert!(path.join("Mod.uplugin").is_file());
    assert!(!state.join("staging").exists());
    assert!(!state.join("backup").exists());
    drop(store);
    fs::remove_dir_all(dir).unwrap();
}
