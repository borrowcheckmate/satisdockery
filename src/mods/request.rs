use semver::VersionReq;
use serde::Serialize;

use crate::Result;

/// The Satisfactory Mod Loader, which every mod requires.
const MOD_LOADER: &str = "SML";

#[derive(Clone, Debug, Serialize)]
pub struct ModRequest {
    #[serde(rename = "modIdOrReference")]
    pub name: String,
    pub version: VersionReq,
}

impl ModRequest {
    /// Request for the mod loader at the oldest version that supports Linux servers.
    pub fn mod_loader() -> Result<Self> {
        Self::at_least(MOD_LOADER, ">=1.0.0")
    }

    pub fn any_version(name: &str) -> Result<Self> {
        Self::at_least(name, ">=0.0.0")
    }

    fn at_least(name: &str, constraint: &str) -> Result<Self> {
        Ok(Self {
            name: name.to_owned(),
            version: VersionReq::parse(constraint)?,
        })
    }
}

/// Parses `Name:constraint` entries, skipping (with a warning) entries without a constraint.
pub fn parse_all(entries: &[String]) -> Result<Vec<ModRequest>> {
    let mut requests = Vec::new();
    for entry in entries {
        let Some((name, constraint)) = entry.split_once(':') else {
            log::warn!("Invalid request provided: {entry}");
            continue;
        };
        requests.push(ModRequest {
            name: name.to_owned(),
            version: VersionReq::parse(constraint)?,
        });
    }
    Ok(requests)
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;

    #[test]
    fn parses_name_and_constraint() {
        let requests = parse_all(&["RefinedPower:>=4.0.44".to_owned()]).unwrap();

        assert_eq!(requests[0].name, "RefinedPower");
        assert!(requests[0].version.matches(&Version::new(4, 1, 0)));
        assert!(!requests[0].version.matches(&Version::new(4, 0, 43)));
    }

    #[test]
    fn skips_entries_without_constraint() {
        assert!(parse_all(&["NoConstraint".to_owned()]).unwrap().is_empty());
    }

    #[test]
    fn rejects_malformed_constraints() {
        assert!(parse_all(&["Mod:not-a-constraint".to_owned()]).is_err());
    }
}
