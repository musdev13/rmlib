use std::path::{Path, PathBuf};

use musutils::fs::config::ConfigError;

use super::dto::MsTokens;
use super::error::AuthError;

const FILE_NAME: &str = "ms.json";

pub(crate) fn default_auth_dir() -> Result<PathBuf, ConfigError> {
    let paths = crate::core::config::Paths::load()?;
    Ok(paths.auth_pathbuf())
}

fn resolve_dir(dir: Option<&Path>) -> Result<PathBuf, AuthError> {
    match dir {
        Some(p) => Ok(p.to_path_buf()),
        None => Ok(default_auth_dir()?),
    }
}

pub fn tokens_file_at(dir: Option<&Path>) -> Result<PathBuf, AuthError> {
    Ok(resolve_dir(dir)?.join(FILE_NAME))
}

pub fn save_tokens_at(dir: Option<&Path>, tokens: &MsTokens) -> Result<(), AuthError> {
    let path = tokens_file_at(dir)?;
    let text = serde_json::to_string_pretty(tokens)?;
    musutils::fs::write_atomic(&path, text)?;
    Ok(())
}

pub fn load_tokens_at(dir: Option<&Path>) -> Result<Option<MsTokens>, AuthError> {
    let path = tokens_file_at(dir)?;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)?;
    match serde_json::from_str::<MsTokens>(&text) {
        Ok(tokens) => Ok(Some(tokens)),
        Err(_) => Ok(None),
    }
}

pub fn clear_tokens_at(dir: Option<&Path>) -> Result<(), AuthError> {
    let path = tokens_file_at(dir)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn save_tokens(tokens: &MsTokens) -> Result<(), AuthError> {
    save_tokens_at(None, tokens)
}

pub fn load_tokens() -> Result<Option<MsTokens>, AuthError> {
    load_tokens_at(None)
}

pub fn clear_tokens() -> Result<(), AuthError> {
    clear_tokens_at(None)
}

pub fn tokens_file() -> Result<PathBuf, AuthError> {
    tokens_file_at(None)
}
