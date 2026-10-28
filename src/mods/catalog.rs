use reqwest::Client;
use semver::{Version, VersionReq};
use serde::Deserialize;
use serde_json::json;

use super::request::ModRequest;
use crate::Result;

/// Endpoint to the ficsit.app GraphQL api (<https://ficsit.app/api-docs>)
const GRAPHQL_API: &str = "https://api.ficsit.app/v2/query";
const RESOLVE_MODS_QUERY: &str = include_str!("../../graphql/resolve_mods.graphql");
const WORKING_STATE: &str = "Works";

#[derive(Clone, Debug, Deserialize)]
pub struct ResolvedMod {
    pub mod_reference: String,
    pub versions: Vec<ModVersion>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModVersion {
    pub version: Version,
    pub game_version: VersionReq,
    pub link: String,
    #[serde(rename = "mod")]
    pub parent: ModInfo,
    pub dependencies: Vec<Dependency>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModInfo {
    pub compatibility: Compatibility,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Compatibility {
    #[serde(rename = "EA")]
    pub early_access: CompatibilityState,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CompatibilityState {
    pub state: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    pub mod_reference: String,
    pub condition: VersionReq,
}

#[derive(Deserialize)]
struct Response {
    data: ResponseData,
}

#[derive(Deserialize)]
struct ResponseData {
    #[serde(rename = "resolveModVersions")]
    resolved_mods: Vec<ResolvedMod>,
}

/// Fetches every version of the requested mods that satisfies their constraints.
pub async fn fetch(requests: &[ModRequest]) -> Result<Vec<ResolvedMod>> {
    let payload = json!({
        "query": RESOLVE_MODS_QUERY,
        "variables": { "constraints": requests },
    });
    let response: Response = Client::new()
        .post(GRAPHQL_API)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(response.data.resolved_mods)
}

/// Drops versions that do not run on `game_build` or are not marked as working.
pub fn retain_compatible(mods: &mut [ResolvedMod], game_build: &Version) {
    for resolved in mods {
        resolved.versions.retain(|version| {
            version.game_version.matches(game_build)
                && version.parent.compatibility.early_access.state == WORKING_STATE
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mod_version(game_version: &str, state: &str) -> ModVersion {
        ModVersion {
            version: Version::new(1, 0, 0),
            game_version: VersionReq::parse(game_version).unwrap(),
            link: String::new(),
            parent: ModInfo {
                compatibility: Compatibility {
                    early_access: CompatibilityState {
                        state: state.to_owned(),
                    },
                },
            },
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn keeps_only_working_versions_matching_the_game_build() {
        let mut mods = vec![ResolvedMod {
            mod_reference: "Example".to_owned(),
            versions: vec![
                mod_version(">=400000", "Works"),
                mod_version(">=500000", "Works"),
                mod_version(">=400000", "Broken"),
            ],
        }];

        retain_compatible(&mut mods, &Version::new(424_242, 0, 0));

        assert_eq!(mods[0].versions.len(), 1);
    }

    #[test]
    fn parses_the_api_response_shape() {
        let body = json!({"data": {"resolveModVersions": [{
            "id": "1", "mod_reference": "Example",
            "versions": [{
                "version": "1.2.3", "game_version": ">=1", "link": "/x", "stability": "release",
                "approved": true,
                "mod": {"compatibility": {"EA": {"state": "Works"}, "EXP": {"state": "Works"}}},
                "dependencies": [{"mod_reference": "SML", "condition": ">=3.0.0"}]
            }]
        }]}});

        let response: Response = serde_json::from_value(body).unwrap();

        assert_eq!(
            response.data.resolved_mods[0].versions[0].dependencies[0].mod_reference,
            "SML"
        );
    }
}
