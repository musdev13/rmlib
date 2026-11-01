use std::path::Path;

use super::dto::ModInfo;
use super::error::ModError;
use super::parse::analyze_jar;

pub fn info_from_file(path: &Path) -> Result<ModInfo, ModError> {
    if !path.exists() {
        return Err(ModError::NotFound(format!(
            "file not found: {}",
            path.display()
        )));
    }
    analyze_jar(path)
}

pub fn info_by_id(mods_dir: &Path, id: &str) -> Result<Option<ModInfo>, ModError> {
    if !mods_dir.exists() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !(name.ends_with(".jar") || name.ends_with(".jar.disabled")) {
            continue;
        }
        if let Ok(info) = analyze_jar(&path)
            && info.id.as_deref() == Some(id)
        {
            return Ok(Some(info));
        }
    }
    Ok(None)
}
