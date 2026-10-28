use crate::{Error, Result};

/// A pinned `DepotDownloader` release for one CPU architecture.
pub struct Build {
    pub url: &'static str,
    /// SHA-256 of the extracted `DepotDownloader` executable.
    pub sha256: &'static str,
}

const X86_64: Build = Build {
    url: "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_3.4.0/DepotDownloader-linux-x64.zip",
    sha256: "d62a1721564bdb96bacd9285bb5f96180a45202e82a9f85c6a88e5e8ee5f992c",
};

const AARCH64: Build = Build {
    url: "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_3.4.0/DepotDownloader-linux-arm64.zip",
    sha256: "07190213d6eb59799a59fb2cc763f52f51a59bfe8ad0f72c36044fff0fce9a09",
};

pub const fn current_build() -> Result<Build> {
    match std::env::consts::ARCH.as_bytes() {
        b"x86_64" => Ok(X86_64),
        b"aarch64" => Ok(AARCH64),
        _ => Err(Error::InvalidArchitecture),
    }
}
