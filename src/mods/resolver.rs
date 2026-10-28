use std::collections::HashMap;

use semver::VersionReq;

use super::{
    Package,
    catalog::{self, ModVersion, ResolvedMod},
    request::ModRequest,
};
use crate::Result;

/// Resolves the requests and all their transitive dependencies to one package per mod.
///
/// Every mod is resolved to the highest version that satisfies the constraints of everything
/// that depends on it.
pub async fn resolve(catalog: Vec<ResolvedMod>, requests: &[ModRequest]) -> Result<Vec<Package>> {
    let mut resolver = Resolver::new(catalog, requests);
    let mut pending: Vec<String> = requests
        .iter()
        .map(|request| request.name.clone())
        .collect();
    while !pending.is_empty() {
        pending = resolver.resolve_round(pending).await?;
    }
    Ok(resolver.into_packages())
}

struct Resolver {
    constraints: HashMap<String, Vec<VersionReq>>,
    versions: HashMap<String, Vec<ModVersion>>,
    selected: HashMap<String, Package>,
}

impl Resolver {
    fn new(catalog: Vec<ResolvedMod>, requests: &[ModRequest]) -> Self {
        let mut resolver = Self {
            constraints: HashMap::new(),
            versions: catalog
                .into_iter()
                .map(|resolved| (resolved.mod_reference, resolved.versions))
                .collect(),
            selected: HashMap::new(),
        };
        for request in requests {
            resolver.constrain(&request.name, request.version.clone());
        }
        resolver
    }

    fn constrain(&mut self, name: &str, constraint: VersionReq) {
        self.constraints
            .entry(name.to_owned())
            .or_default()
            .push(constraint);
    }

    /// Selects a version for every pending mod and returns the dependencies that follow from it.
    async fn resolve_round(&mut self, pending: Vec<String>) -> Result<Vec<String>> {
        let mut next = Vec::new();
        for name in pending {
            next.extend(self.resolve_one(&name).await?);
        }
        Ok(next)
    }

    async fn resolve_one(&mut self, name: &str) -> Result<Vec<String>> {
        if self.selected.contains_key(name) || !self.ensure_versions(name).await? {
            return Ok(Vec::new());
        }
        Ok(self
            .choose_version(name)
            .map(|chosen| self.adopt(name, chosen))
            .unwrap_or_default())
    }

    /// Dependencies are not part of the initial query, so they are fetched when first seen.
    async fn ensure_versions(&mut self, name: &str) -> Result<bool> {
        if self.versions.contains_key(name) {
            return Ok(true);
        }
        log::info!("Fetching additional mod: {name}");
        let fetched = catalog::fetch(&[ModRequest::any_version(name)?]).await?;
        let Some(resolved) = fetched.into_iter().next() else {
            log::warn!("Could not fetch mod: {name}");
            return Ok(false);
        };
        self.versions.insert(name.to_owned(), resolved.versions);
        Ok(true)
    }

    fn choose_version(&self, name: &str) -> Option<ModVersion> {
        let constraints = self.constraints.get(name).map_or(&[][..], Vec::as_slice);
        let chosen = self
            .versions
            .get(name)?
            .iter()
            .filter(|candidate| {
                constraints
                    .iter()
                    .all(|constraint| constraint.matches(&candidate.version))
            })
            .max_by(|a, b| a.version.cmp(&b.version));
        if chosen.is_none() {
            log::warn!(
                "No versions found for {name} that are compatible with the current game version"
            );
        }
        chosen.cloned()
    }

    /// Records `chosen` as the package for `name` and returns the mods it depends on.
    fn adopt(&mut self, name: &str, chosen: ModVersion) -> Vec<String> {
        self.selected.insert(
            name.to_owned(),
            Package {
                name: name.to_owned(),
                version: chosen.version,
                link: chosen.link.replace("Windows", "LinuxServer"),
            },
        );
        let mut dependencies = Vec::new();
        for dependency in chosen.dependencies {
            self.constrain(&dependency.mod_reference, dependency.condition);
            dependencies.push(dependency.mod_reference);
        }
        dependencies
    }

    fn into_packages(self) -> Vec<Package> {
        let mut packages: Vec<Package> = self.selected.into_values().collect();
        packages.sort_by(|a, b| a.name.cmp(&b.name));
        packages
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;
    use crate::mods::catalog::{Compatibility, CompatibilityState, Dependency, ModInfo};

    fn version(number: &str, dependencies: Vec<Dependency>) -> ModVersion {
        ModVersion {
            version: Version::parse(number).unwrap(),
            game_version: VersionReq::STAR,
            link: format!("/Windows/{number}"),
            parent: ModInfo {
                compatibility: Compatibility {
                    early_access: CompatibilityState {
                        state: "Works".to_owned(),
                    },
                },
            },
            dependencies,
        }
    }

    fn catalog_entry(name: &str, versions: Vec<ModVersion>) -> ResolvedMod {
        ResolvedMod {
            mod_reference: name.to_owned(),
            versions,
        }
    }

    fn request(name: &str, constraint: &str) -> ModRequest {
        ModRequest {
            name: name.to_owned(),
            version: VersionReq::parse(constraint).unwrap(),
        }
    }

    #[tokio::test]
    async fn picks_the_highest_version_and_follows_dependencies() {
        let dependency = Dependency {
            mod_reference: "SML".to_owned(),
            condition: VersionReq::parse(">=1.0.0").unwrap(),
        };
        let catalog = vec![
            catalog_entry(
                "Mod",
                vec![version("1.0.0", vec![]), version("2.0.0", vec![dependency])],
            ),
            catalog_entry(
                "SML",
                vec![version("1.0.0", vec![]), version("3.0.0", vec![])],
            ),
        ];

        let packages = resolve(catalog, &[request("Mod", ">=1.0.0")])
            .await
            .unwrap();

        let resolved: Vec<_> = packages
            .iter()
            .map(|package| (package.name.as_str(), package.version.to_string()))
            .collect();
        assert_eq!(
            resolved,
            [("Mod", "2.0.0".to_owned()), ("SML", "3.0.0".to_owned())]
        );
    }

    #[tokio::test]
    async fn rewrites_download_links_to_the_linux_server_build() {
        let catalog = vec![catalog_entry("Mod", vec![version("1.0.0", vec![])])];

        let packages = resolve(catalog, &[request("Mod", ">=1.0.0")])
            .await
            .unwrap();

        assert_eq!(packages[0].link, "/LinuxServer/1.0.0");
    }

    #[tokio::test]
    async fn skips_mods_without_a_satisfying_version() {
        let catalog = vec![catalog_entry("Mod", vec![version("1.0.0", vec![])])];

        let packages = resolve(catalog, &[request("Mod", ">=2.0.0")])
            .await
            .unwrap();

        assert!(packages.is_empty());
    }
}
