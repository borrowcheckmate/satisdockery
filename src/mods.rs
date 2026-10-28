mod catalog;
mod installer;
mod package;
mod request;
mod resolver;

use semver::Version;

use crate::Result;

pub use installer::install;
pub use package::Package;

/// Resolves the requested mods, plus everything they depend on, to downloadable packages.
///
/// # Errors
/// Fails when a request is malformed or the mod catalog cannot be queried.
pub async fn resolve_packages(requested: &[String], game_build: Version) -> Result<Vec<Package>> {
    if requested.is_empty() {
        log::info!("No mods provided so will skip processing.");
        return Ok(Vec::new());
    }

    log::info!("Processing mods...");
    let mut requests = request::parse_all(requested)?;
    requests.push(request::ModRequest::mod_loader()?);
    log::info!(
        "Mods requested: \n{}",
        serde_json::to_string_pretty(&requests)?
    );

    let mut catalog = catalog::fetch(&requests).await?;
    catalog::retain_compatible(&mut catalog, &game_build);
    resolver::resolve(catalog, &requests).await
}
