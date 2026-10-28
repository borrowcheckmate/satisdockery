use semver::Version;

/// A concrete mod version chosen for installation.
#[derive(Clone, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub link: String,
}
