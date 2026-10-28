use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde::Deserialize;

use crate::Result;

/// Contents of the `FactoryServer-Linux-Shipping.version` file shipped with the server.
#[derive(Clone, Debug, Deserialize)]
pub struct GameVersion {
    #[serde(rename = "MajorVersion")]
    pub major: u32,
    #[serde(rename = "MinorVersion")]
    pub minor: u32,
    #[serde(rename = "PatchVersion")]
    pub patch: u32,
    #[serde(rename = "Changelist")]
    pub changelist: u64,
    #[serde(rename = "CompatibleChangelist")]
    pub compatible_changelist: u64,
    #[serde(rename = "IsLicenseeVersion")]
    pub is_licensee_version: u32,
    #[serde(rename = "IsPromotedBuild")]
    pub promoted_build: u32,
    #[serde(rename = "BranchName")]
    pub branch_name: String,
    #[serde(rename = "BuildId")]
    pub build_id: VersionReq,
    #[serde(rename = "GameVersion")]
    pub game_version: String,
}

impl GameVersion {
    /// Reads the version file of the installation at `install_dir`.
    ///
    /// # Errors
    /// Fails when the file is missing or is not a valid version file.
    pub async fn read(install_dir: &Path) -> Result<Self> {
        let version_file = binaries_dir(install_dir).join("FactoryServer-Linux-Shipping.version");
        let contents = tokio::fs::read_to_string(version_file).await?;
        Ok(serde_json::from_str(&contents)?)
    }

    /// Mods declare compatibility as a semver range over the changelist number.
    #[must_use]
    pub const fn mod_build_requirement(&self) -> Version {
        Version::new(self.changelist, 0, 0)
    }
}

#[must_use]
pub fn binaries_dir(install_dir: &Path) -> PathBuf {
    install_dir.join("Engine/Binaries/Linux")
}
