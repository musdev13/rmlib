use std::path::{Path, PathBuf};

use super::dto::ResourcepackInfo;
use super::error::RpError;
use super::parse::analyze_zip;

pub fn resourcepacks_dir(game_path: &Path) -> PathBuf {
    game_path.join("resourcepacks")
}

pub fn scan_dir(dir: &Path) -> Result<Vec<ResourcepackInfo>, RpError> {
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
        let is_zip = name.ends_with(".zip") || name.ends_with(".zip.disabled");
        if !is_zip {
            continue;
        }
        match analyze_zip(&path) {
            Ok(info) => out.push(info),
            Err(_) => {
                let filename = name.to_string();
                let enabled = !filename.ends_with(".disabled");
                let stem = filename
                    .strip_suffix(".disabled")
                    .unwrap_or(&filename)
                    .to_string();
                out.push(ResourcepackInfo {
                    path,
                    filename: stem,
                    enabled,
                    size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                    ..Default::default()
                });
            }
        }
    }
    out.sort_by_key(|a| a.filename.to_lowercase());
    Ok(out)
}

pub fn scan(game_path: &Path) -> Result<Vec<ResourcepackInfo>, RpError> {
    scan_dir(&resourcepacks_dir(game_path))
}
