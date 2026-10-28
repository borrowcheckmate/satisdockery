use std::{path::PathBuf, process::ExitStatus};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Architecture is not compatible with Satisfactory.")]
    InvalidArchitecture,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Hash mismatch: expected {expected}, computed {computed}")]
    HashMismatch { expected: String, computed: String },
    #[error("Version error: {0}")]
    Version(#[from] semver::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{program} exited unsuccessfully: {status}")]
    CommandFailed { program: String, status: ExitStatus },
    #[error("Invalid mod reference: {0}")]
    InvalidModReference(String),
    #[error("Path is not valid UTF-8: {}", .0.display())]
    NonUtf8Path(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
