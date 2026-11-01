use std::path::Path;

use super::dto::ResourcepackInfo;
use super::error::RpError;
use super::source::{FallbackPolicy, Priority};

pub async fn scan_priority_dir(
    dir: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Vec<ResourcepackInfo>, RpError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !(name.ends_with(".zip") || name.ends_with(".zip.disabled")) {
            continue;
        }
        match super::info_priority::info_priority(&path, priority, fallback).await {
            Ok(Some(info)) => out.push(info),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }
    out.sort_by_key(|a| a.filename.to_lowercase());
    Ok(out)
}

pub async fn scan_priority(
    game_path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Vec<ResourcepackInfo>, RpError> {
    let dir = super::scan::resourcepacks_dir(game_path);
    scan_priority_dir(&dir, priority, fallback).await
}
