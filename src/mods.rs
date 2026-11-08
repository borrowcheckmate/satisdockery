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
        return Ok(Vec::new());
    }

    log::info!("Resolving mods for game build {game_build}");
    let mut requests = request::parse_all(requested)?;
    requests.push(request::ModRequest::mod_loader()?);
    for request in &requests {
        log::debug!("Requested mod {} {}", request.name, request.version);
    }

    let mut catalog = catalog::fetch(&requests).await?;
    catalog::retain_compatible(&mut catalog, &game_build);
    resolver::resolve(catalog, &requests).await
}
