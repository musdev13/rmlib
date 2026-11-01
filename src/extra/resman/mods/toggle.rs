use std::path::{Path, PathBuf};

use super::error::ModError;

pub fn is_enabled(path: &Path) -> bool {
    !path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with(".disabled"))
        .unwrap_or(false)
}

fn enabled_path(path: &Path) -> Result<PathBuf, ModError> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ModError::InvalidArgument("path has no filename".to_string()))?;
    let new_name = name.trim_end_matches(".disabled");
    Ok(path.with_file_name(new_name))
}

fn disabled_path(path: &Path) -> Result<PathBuf, ModError> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ModError::InvalidArgument("path has no filename".to_string()))?;
    if name.ends_with(".disabled") {
        return Ok(path.to_path_buf());
    }
    Ok(path.with_file_name(format!("{name}.disabled")))
}

pub fn enable(path: &Path) -> Result<PathBuf, ModError> {
    let new = enabled_path(path)?;
    if new != path {
        std::fs::rename(path, &new)?;
    }
    Ok(new)
}

pub fn disable(path: &Path) -> Result<PathBuf, ModError> {
    let new = disabled_path(path)?;
    if new != path {
        std::fs::rename(path, &new)?;
    }
    Ok(new)
}

pub fn toggle(path: &Path) -> Result<(PathBuf, bool), ModError> {
    if is_enabled(path) {
        let p = disable(path)?;
        Ok((p, false))
    } else {
        let p = enable(path)?;
        Ok((p, true))
    }
}
