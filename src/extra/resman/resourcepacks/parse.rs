use std::io::Read;
use std::path::Path;

use serde_json::Value;

use super::dto::ResourcepackInfo;
use super::error::RpError;
use super::icon::save_icon;

pub(crate) fn read_entry<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<Vec<u8>> {
    let mut f = zip.by_name(name).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

pub(crate) fn analyze_zip(path: &Path) -> Result<ResourcepackInfo, RpError> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let names: Vec<String> = zip
        .file_names()
        .filter_map(|r| r.ok().map(|s| s.into_owned()))
        .collect();
    let has = |n: &str| names.iter().any(|x| x == n);

    if !has("pack.mcmeta") {
        return Err(RpError::InvalidArgument(
            "pack.mcmeta not found in ZIP".to_string(),
        ));
    }

    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let enabled = !filename.ends_with(".disabled");
    let stem = filename
        .strip_suffix(".disabled")
        .unwrap_or(&filename)
        .to_string();
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let bytes = read_entry(&mut zip, "pack.mcmeta")
        .ok_or_else(|| RpError::NotFound("pack.mcmeta".to_string()))?;
    let mcmeta: Value = serde_json::from_slice(&bytes)?;

    let pack = mcmeta.get("pack");
    let description = pack
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str())
        .map(str::to_string);

    let read_u64 = |key: &str| pack.and_then(|p| p.get(key)).and_then(|v| v.as_u64());

    let pack_format = read_u64("pack_format")
        .or_else(|| read_u64("min_format"))
        .or_else(|| read_u64("max_format"))
        .map(|n| n as u32);

    let supported_formats = pack
        .and_then(|p| p.get("supported_formats"))
        .cloned()
        .or_else(|| {
            let min = read_u64("min_format");
            let max = read_u64("max_format");
            match (min, max) {
                (Some(a), Some(b)) => {
                    Some(serde_json::json!({"min_inclusive": a, "max_inclusive": b}))
                }
                (Some(a), None) => Some(serde_json::json!({"min_inclusive": a})),
                (None, Some(b)) => Some(serde_json::json!({"max_inclusive": b})),
                _ => None,
            }
        });

    let mut icon = None;
    if has("pack.png")
        && let Some(img) = read_entry(&mut zip, "pack.png")
    {
        icon = Some(save_icon(&stem, &img)?);
    }

    Ok(ResourcepackInfo {
        path: path.to_path_buf(),
        filename: stem,
        enabled,
        size,
        id: None,
        description,
        pack_format,
        supported_formats,
        icon,
    })
}
