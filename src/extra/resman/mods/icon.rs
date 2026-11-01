use std::path::PathBuf;

use super::error::ModError;

pub fn icons_dir() -> PathBuf {
    std::env::temp_dir().join("rmlib_icons")
}

pub fn save_icon(id: &str, bytes: &[u8]) -> Result<PathBuf, ModError> {
    let dir = icons_dir();
    std::fs::create_dir_all(&dir)?;
    let safe = id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let path = dir.join(format!("{safe}.png"));
    std::fs::write(&path, bytes)?;
    Ok(path)
}
