use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use semver::Version;
use serde::{Deserialize, Serialize};

use super::{ensure_valid_directory_name, regular_file};
use crate::Result;

pub(super) const STATE_DIR: &str = ".satisdockery";
const RECORD: &str = ".satisdockery-installed.json";

#[derive(Deserialize, Serialize)]
pub(super) struct Record {
    pub(super) name: String,
    pub(super) version: Version,
    pub(super) game_feature: bool,
}

impl Record {
    pub(super) fn read(directory: &Path) -> Result<Option<Self>> {
        let path = directory.join(RECORD);
        if !regular_file(&path)? {
            return Ok(None);
        }
        let record: Self = serde_json::from_slice(&fs::read(path)?)?;
        ensure_valid_directory_name(&record.name)?;
        Ok(Some(record))
    }

    pub(super) fn write(&self, directory: &Path) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(RECORD))?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}

pub(super) struct Store {
    mods: PathBuf,
    state: PathBuf,
    #[cfg(unix)]
    _lock: nix::fcntl::Flock<File>,
    #[cfg(not(unix))]
    _lock: File,
}

impl Store {
    pub(super) fn open(install_dir: &Path, needed: bool) -> Result<Option<Self>> {
        let factory = install_dir.join("FactoryGame");
        let mods = factory.join("Mods");
        let state = mods.join(STATE_DIR);
        if !needed && !directory_exists(&state)? {
            return Ok(None);
        }
        for dir in [&factory, &mods, &state] {
            create_directory(dir)?;
        }
        directory_exists(&mods.join("GameFeatures"))?;
        let lock_path = state.join("lock");
        let lock = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        #[cfg(unix)]
        let lock = nix::fcntl::Flock::lock(lock, nix::fcntl::FlockArg::LockExclusiveNonblock)
            .map_err(|(_, error)| {
                io::Error::other(format!("Cannot lock the mod installation: {error}"))
            })?;
        Ok(Some(Self {
            mods,
            state,
            _lock: lock,
        }))
    }

    pub(super) fn staging(&self) -> PathBuf {
        self.state.join("staging")
    }

    pub(super) fn recover(&self) -> Result<()> {
        remove_directory(&self.state.join("discard"))?;
        let backup = self.state.join("backup");
        if directory_exists(&backup)? {
            let record = Record::read(&backup)?
                .ok_or_else(|| io::Error::other("Missing mod backup record"))?;
            self.recover_backup(&backup, &record)?;
        }
        remove_directory(&self.staging())?;
        Ok(())
    }

    fn recover_backup(&self, backup: &Path, record: &Record) -> Result<()> {
        let Some(current) = self.find(&record.name)? else {
            fs::rename(backup, self.location(&record.name, record.game_feature))?;
            return Ok(());
        };
        if Record::read(&current)?.is_none_or(|owner| owner.name != record.name) {
            return Err(
                io::Error::other(format!("Cannot restore backup for mod {}", record.name)).into(),
            );
        }
        self.discard(backup)?;
        Ok(())
    }

    pub(super) fn find(&self, name: &str) -> Result<Option<PathBuf>> {
        let normal = self.location(name, false);
        let feature = self.location(name, true);
        match (directory_exists(&normal)?, directory_exists(&feature)?) {
            (true, false) => Ok(Some(normal)),
            (false, true) => Ok(Some(feature)),
            (false, false) => Ok(None),
            (true, true) => Err(io::Error::other(format!(
                "Mod {name} exists in both plugin directories"
            ))
            .into()),
        }
    }

    pub(super) fn publish(
        &self,
        staging: &Path,
        record: &Record,
        existing: Option<&Path>,
    ) -> Result<()> {
        let target = self.location(&record.name, record.game_feature);
        if record.game_feature {
            create_directory(&self.mods.join("GameFeatures"))?;
        }
        let backup = self.state.join("backup");
        if let Some(old) = existing {
            fs::rename(old, &backup)?;
        }
        let published = fs::rename(staging, target);
        if published.is_err()
            && let Some(old) = existing
        {
            fs::rename(&backup, old)?;
        }
        published?;
        self.discard(&backup)?;
        Ok(())
    }

    pub(super) fn remove_unrequested(&self, names: &HashSet<&str>) -> Result<()> {
        self.remove_from(&self.mods, names)?;
        self.remove_from(&self.mods.join("GameFeatures"), names)
    }

    fn remove_from(&self, directory: &Path, names: &HashSet<&str>) -> Result<()> {
        if !directory_exists(directory)? {
            return Ok(());
        }
        let entries = fs::read_dir(directory)?.collect::<io::Result<Vec<_>>>()?;
        for entry in &entries {
            self.remove_if_owned(entry, names)?;
        }
        Ok(())
    }

    fn remove_if_owned(&self, entry: &fs::DirEntry, names: &HashSet<&str>) -> Result<()> {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Ok(());
        };
        if names.contains(name)
            || ensure_valid_directory_name(name).is_err()
            || !entry.file_type()?.is_dir()
        {
            return Ok(());
        }
        let path = entry.path();
        if let Some(record) = Record::read(&path)?
            && path == self.location(&record.name, record.game_feature)
        {
            self.discard(&path)?;
            log::info!("Removed mod {} {}", record.name, record.version);
        }
        Ok(())
    }

    fn discard(&self, path: &Path) -> io::Result<()> {
        if directory_exists(path)? {
            let discard = self.state.join("discard");
            fs::rename(path, &discard)?;
            remove_directory(&discard)?;
        }
        Ok(())
    }

    fn location(&self, name: &str, game_feature: bool) -> PathBuf {
        if game_feature {
            self.mods.join("GameFeatures").join(name)
        } else {
            self.mods.join(name)
        }
    }
}

fn create_directory(path: &Path) -> io::Result<()> {
    match fs::create_dir(path) {
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists && directory_exists(path)? => {
            Ok(())
        }
        result => result,
    }
}

fn remove_directory(path: &Path) -> io::Result<()> {
    if directory_exists(path)? {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn directory_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(io::Error::other(format!(
            "Expected a directory at {}",
            path.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
