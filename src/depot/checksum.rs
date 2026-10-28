use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{Error, Result};

/// Verifies that the SHA-256 of the file at `path` equals the hex digest `expected`.
pub fn verify(path: &Path, expected: &str) -> Result<()> {
    let computed = hex::encode(Sha256::digest(fs::read(path)?));
    if computed == expected {
        Ok(())
    } else {
        Err(Error::HashMismatch {
            expected: expected.to_owned(),
            computed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn accepts_matching_digest_of_the_whole_file() {
        let directory = tempdir("accepts");
        let file = directory.join("fixture");
        fs::write(&file, b"abc").unwrap();

        let result = verify(&file, ABC_SHA256);
        fs::remove_dir_all(directory).unwrap();

        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn rejects_mismatching_digest() {
        let directory = tempdir("rejects");
        let file = directory.join("fixture");
        fs::write(&file, b"abc").unwrap();

        let result = verify(&file, EMPTY_SHA256);
        fs::remove_dir_all(directory).unwrap();

        assert!(matches!(result, Err(Error::HashMismatch { .. })));
    }

    fn tempdir(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "satisdockery-checksum-{name}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }
}
