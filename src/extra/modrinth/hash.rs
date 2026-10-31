use std::path::Path;

use sha1::Digest as _;
use sha2::Digest as _;

use super::error::ModrinthError;

pub fn sha1_file(path: &Path) -> Result<String, ModrinthError> {
    let bytes = std::fs::read(path)?;
    let mut hasher = sha1::Sha1::new();
    hasher.update(&bytes);
    Ok(hex(hasher.finalize().as_slice()))
}

pub fn sha512_file(path: &Path) -> Result<String, ModrinthError> {
    let bytes = std::fs::read(path)?;
    let mut hasher = sha2::Sha512::new();
    hasher.update(&bytes);
    Ok(hex(hasher.finalize().as_slice()))
}

pub fn sha1_bytes(bytes: &[u8]) -> String {
    let mut hasher = sha1::Sha1::new();
    hasher.update(bytes);
    hex(hasher.finalize().as_slice())
}

pub fn sha512_bytes(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha512::new();
    hasher.update(bytes);
    hex(hasher.finalize().as_slice())
}

pub fn verify_sha1(path: &Path, expected: &str) -> Result<(), ModrinthError> {
    let actual = sha1_file(path)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(ModrinthError::HashMismatch {
            expected: expected.to_string(),
            actual,
        });
    }
    Ok(())
}

pub fn verify_sha512(path: &Path, expected: &str) -> Result<(), ModrinthError> {
    let actual = sha512_file(path)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(ModrinthError::HashMismatch {
            expected: expected.to_string(),
            actual,
        });
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}
