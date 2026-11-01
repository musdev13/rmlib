use std::path::{Path, PathBuf};

use super::dto::ModInfo;
use super::error::ModError;
use super::parse::analyze_jar;

pub fn mods_dir(game_path: &Path) -> PathBuf {
    game_path.join("mods")
}

pub fn scan_dir(mods_dir: &Path) -> Result<Vec<ModInfo>, ModError> {
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let is_jar = name.ends_with(".jar") || name.ends_with(".jar.disabled");
        if !is_jar {
            continue;
        }
        match analyze_jar(&path) {
            Ok(info) => out.push(info),
            Err(e) => {
                let filename = name.to_string();
                let enabled = !filename.ends_with(".disabled");
                let stem = filename
                    .strip_suffix(".disabled")
                    .unwrap_or(&filename)
                    .to_string();
                out.push(ModInfo {
                    path,
                    filename: stem,
                    enabled,
                    size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                    loader: None,
                    ..Default::default()
                });
                let _ = e;
            }
        }
    }
    out.sort_by_key(|a| a.filename.to_lowercase());
    Ok(out)
}

pub fn scan(game_path: &Path) -> Result<Vec<ModInfo>, ModError> {
    scan_dir(&mods_dir(game_path))
}
