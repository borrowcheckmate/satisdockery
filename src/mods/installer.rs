mod store;
#[cfg(test)]
mod tests;

use std::{
    collections::HashSet,
    fs,
    future::Future,
    io::{self, Cursor},
    path::Path,
};

use semver::Version;
use serde::Deserialize;

use super::Package;
use crate::{Error, Result};

use store::{Record, Store};

/// Endpoint to the ficsit.app REST API.
const REST_API: &str = "https://api.ficsit.app/";

#[derive(Deserialize)]
struct PluginDescriptor {
    #[serde(default, rename = "GameFeature")]
    game_feature: bool,
    #[serde(rename = "VersionName")]
    version_name: Option<String>,
}

/// Reconciles packages with the installed mods owned by `SatisDockery`.
///
/// # Errors
/// Fails when a package cannot be installed or an owned mod cannot be removed safely.
pub async fn install(install_dir: &Path, packages: &[Package]) -> Result<()> {
    let client = reqwest::Client::new();
    reconcile(install_dir, packages, |link: String| {
        let url = format!("{REST_API}{}", link.trim_start_matches('/'));
        let client = &client;
        async move {
            Ok(client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?)
        }
    })
    .await
}

async fn reconcile<B: AsRef<[u8]>, F: Future<Output = Result<B>>>(
    install_dir: &Path,
    packages: &[Package],
    mut fetch: impl FnMut(String) -> F,
) -> Result<()> {
    let names = validate_packages(packages)?;
    let Some(store) = Store::open(install_dir, !packages.is_empty())? else {
        return Ok(());
    };
    store.recover()?;
    for package in packages {
        let existing = store.find(&package.name)?;
        if needs_install(package, existing.as_deref())? {
            let archive = fetch(package.link.clone()).await?;
            install_package(&store, package, existing.as_deref(), archive.as_ref())?;
            log::info!("Installed mod {} {}", package.name, package.version);
        } else {
            log::info!("Kept mod {} {}", package.name, package.version);
        }
    }
    store.remove_unrequested(&names)
}

fn validate_packages(packages: &[Package]) -> Result<HashSet<&str>> {
    let mut names = HashSet::new();
    for package in packages {
        ensure_valid_directory_name(&package.name)?;
        if !names.insert(package.name.as_str()) {
            return Err(io::Error::other(format!("Duplicate mod {}", package.name)).into());
        }
    }
    Ok(names)
}

fn needs_install(package: &Package, existing: Option<&Path>) -> Result<bool> {
    let Some(path) = existing else {
        return Ok(true);
    };
    let descriptor = path.join(format!("{}.uplugin", package.name));
    if let Some(record) = Record::read(path)? {
        if record.name != package.name {
            return Err(io::Error::other(format!(
                "Mod {} conflicts with the installed mod {}",
                package.name, record.name
            ))
            .into());
        }
        return Ok(record.version != package.version || !regular_file(&descriptor)?);
    }
    if regular_file(&descriptor)? {
        let descriptor: PluginDescriptor = serde_json::from_slice(&fs::read(descriptor)?)?;
        if descriptor
            .version_name
            .as_deref()
            .and_then(|version| Version::parse(version).ok())
            == Some(package.version.clone())
        {
            return Ok(false);
        }
    }
    Err(io::Error::other(format!(
        "Move the unmanaged mod {} out of FactoryGame/Mods before installing version {}",
        package.name, package.version
    ))
    .into())
}

fn install_package(
    store: &Store,
    package: &Package,
    existing: Option<&Path>,
    contents: &[u8],
) -> Result<()> {
    let mut archive = zip::ZipArchive::new(Cursor::new(contents))?;
    let descriptor_name = format!("{}.uplugin", package.name);
    let descriptor: PluginDescriptor = serde_json::from_reader(archive.by_name(&descriptor_name)?)?;
    let staging = store.staging();
    archive.extract(&staging)?;
    if !regular_file(&staging.join(descriptor_name))? {
        return Err(
            io::Error::other("The extracted plugin descriptor must be a regular file").into(),
        );
    }
    let record = Record {
        name: package.name.clone(),
        version: package.version.clone(),
        game_feature: descriptor.game_feature,
    };
    record.write(&staging)?;
    store.publish(&staging, &record, existing)
}

/// Mod references become directory names, so references from API metadata must not escape.
fn ensure_valid_directory_name(name: &str) -> Result<()> {
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.eq_ignore_ascii_case("GameFeatures")
        || name.eq_ignore_ascii_case(store::STATE_DIR)
        || name.contains(['/', '\\', ':', '\0'])
    {
        return Err(Error::InvalidModReference(name.to_owned()));
    }
    Ok(())
}

fn regular_file(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
