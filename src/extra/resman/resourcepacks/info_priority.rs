use std::path::Path;

use super::dto::ResourcepackInfo;
use super::error::RpError;
use super::source::{FallbackPolicy, Field, Priority, Source};

pub async fn info_priority(
    path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Option<ResourcepackInfo>, RpError> {
    let local_allowed = priority.includes(Source::Local);

    for src in priority.sources() {
        match src {
            Source::Local => {
                if let Ok(info) = super::info::info_from_file(path) {
                    return Ok(Some(info));
                }
            }
            Source::Modrinth => {
                if let Some(mut info) = try_modrinth(path).await {
                    if local_allowed && let Ok(local) = super::info::info_from_file(path) {
                        prefer_local(&mut info, &local);
                        fill_missing(&mut info, &local);
                    }
                    return Ok(Some(info));
                }
            }
        }
    }

    match fallback {
        FallbackPolicy::Skip => Ok(None),
        FallbackPolicy::Placeholder => Ok(Some(placeholder(path)?)),
        FallbackPolicy::Fields(fields) => Ok(Some(fallback_fields(path, fields)?)),
    }
}

fn prefer_local(target: &mut ResourcepackInfo, local: &ResourcepackInfo) {
    if local.description.is_some() {
        target.description = local.description.clone();
    }
    if local.pack_format.is_some() {
        target.pack_format = local.pack_format;
    }
    if local.supported_formats.is_some() {
        target.supported_formats = local.supported_formats.clone();
    }
    if local.icon.is_some() {
        target.icon = local.icon.clone();
    }
}

fn fill_missing(target: &mut ResourcepackInfo, local: &ResourcepackInfo) {
    if target.id.is_none() {
        target.id = local.id.clone();
    }
    if target.description.is_none() {
        target.description = local.description.clone();
    }
    if target.pack_format.is_none() {
        target.pack_format = local.pack_format;
    }
    if target.supported_formats.is_none() {
        target.supported_formats = local.supported_formats.clone();
    }
    if target.icon.is_none() {
        target.icon = local.icon.clone();
    }
}

fn placeholder(path: &Path) -> Result<ResourcepackInfo, RpError> {
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
    Ok(ResourcepackInfo {
        path: path.to_path_buf(),
        filename: stem,
        enabled,
        size,
        ..Default::default()
    })
}

fn fallback_fields(path: &Path, fields: &[Field]) -> Result<ResourcepackInfo, RpError> {
    let mut info = placeholder(path)?;
    if let Ok(local) = super::info::info_from_file(path) {
        for f in fields {
            match f {
                Field::Id => info.id = local.id.clone(),
                Field::Description => info.description = local.description.clone(),
                Field::PackFormat => info.pack_format = local.pack_format,
                Field::SupportedFormats => info.supported_formats = local.supported_formats.clone(),
                Field::Icon => info.icon = local.icon.clone(),
                Field::Path | Field::Enabled | Field::Filename | Field::Size => {}
            }
        }
    }
    Ok(info)
}

#[cfg(feature = "modrinth")]
async fn try_modrinth(path: &Path) -> Option<ResourcepackInfo> {
    let hash = crate::extra::modrinth::hash::sha1_file(path).ok()?;
    let project = crate::extra::modrinth::project::get_project_by_hash(&hash)
        .await
        .ok()?;
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

    Some(ResourcepackInfo {
        path: path.to_path_buf(),
        filename: stem,
        enabled,
        size,
        id: Some(project.slug),
        description: Some(project.description),
        pack_format: None,
        supported_formats: None,
        icon: None,
    })
}

#[cfg(not(feature = "modrinth"))]
async fn try_modrinth(_path: &Path) -> Option<ResourcepackInfo> {
    None
}
