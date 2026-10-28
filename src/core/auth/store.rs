use std::path::PathBuf;

use musutils::fs::config::ConfigError;

use super::dto::MsTokens;
use super::error::AuthError;

const FILE_NAME: &str = "ms.json";

pub(crate) fn auth_dir() -> Result<PathBuf, ConfigError> {
    let paths = crate::core::config::Paths::load()?;
    Ok(paths.auth_pathbuf())
}

pub(crate) fn tokens_path() -> Result<PathBuf, ConfigError> {
    Ok(auth_dir()?.join(FILE_NAME))
}

pub fn save_tokens(tokens: &MsTokens) -> Result<(), AuthError> {
    let path = tokens_path()?;
    let text = serde_json::to_string_pretty(tokens)?;
    musutils::fs::write_atomic(&path, text)?;
    Ok(())
}

pub fn load_tokens() -> Result<Option<MsTokens>, AuthError> {
    let path = tokens_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)?;
    match serde_json::from_str::<MsTokens>(&text) {
        Ok(tokens) => Ok(Some(tokens)),
        Err(_) => Ok(None),
    }
}

pub fn clear_tokens() -> Result<(), AuthError> {
    let path = tokens_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn tokens_file() -> Result<PathBuf, AuthError> {
    Ok(tokens_path()?)
}
