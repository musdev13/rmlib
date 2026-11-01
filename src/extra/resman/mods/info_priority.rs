use std::path::Path;

use super::dto::ModInfo;
use super::error::ModError;
use super::source::{FallbackPolicy, Field, Priority, Source};

pub async fn info_priority(
    path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Option<ModInfo>, ModError> {
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
                        prefer_local_for_identity(&mut info, &local);
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

fn prefer_local_for_identity(target: &mut ModInfo, local: &ModInfo) {
    if local.version.is_some() {
        target.version = local.version.clone();
    }
    if local.loader.is_some() {
        target.loader = local.loader;
    }
    if local.minecraft.is_some() {
        target.minecraft = local.minecraft.clone();
    }
}

fn fill_missing(target: &mut ModInfo, local: &ModInfo) {
    if target.id.is_none() {
        target.id = local.id.clone();
    }
    if target.name.is_none() {
        target.name = local.name.clone();
    }
    if target.description.is_none() {
        target.description = local.description.clone();
    }
    if target.authors.is_none() {
        target.authors = local.authors.clone();
    }
    if target.license.is_none() {
        target.license = local.license.clone();
    }
    if target.icon.is_none() {
        target.icon = local.icon.clone();
    }
    if target.dependencies.is_empty() {
        target.dependencies = local.dependencies.clone();
    }
}

fn placeholder(path: &Path) -> Result<ModInfo, ModError> {
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
    Ok(ModInfo {
        path: path.to_path_buf(),
        filename: stem,
        enabled,
        size,
        ..Default::default()
    })
}

fn fallback_fields(path: &Path, fields: &[Field]) -> Result<ModInfo, ModError> {
    let mut info = placeholder(path)?;
    if let Ok(local) = super::info::info_from_file(path) {
        for f in fields {
            match f {
                Field::Id => info.id = local.id.clone(),
                Field::Name => info.name = local.name.clone(),
                Field::Version => info.version = local.version.clone(),
                Field::Description => info.description = local.description.clone(),
                Field::Authors => info.authors = local.authors.clone(),
                Field::License => info.license = local.license.clone(),
                Field::Minecraft => info.minecraft = local.minecraft.clone(),
                Field::Dependencies => info.dependencies = local.dependencies.clone(),
                Field::Icon => info.icon = local.icon.clone(),
                Field::Loader => info.loader = local.loader,
                Field::Path | Field::Enabled | Field::Filename | Field::Size | Field::FabricApi => {
                }
            }
        }
    }
    Ok(info)
}

#[cfg(feature = "modrinth")]
async fn try_modrinth(path: &Path) -> Option<ModInfo> {
    use crate::extra::modrinth as mr;

    let hash = mr::hash::sha1_file(path).ok()?;

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

    let version = mr::version::get_version_by_hash(&hash).await.ok();

    let project = match version.as_ref() {
        Some(v) => mr::project::get_project(&v.project_id).await.ok(),
        None => mr::project::get_project_by_hash(&hash).await.ok(),
    }?;

    let version_number = version.as_ref().map(|v| v.version_number.clone());
    let minecraft = version
        .as_ref()
        .map(|v| v.game_versions.clone())
        .filter(|v| !v.is_empty());
    let loader = version
        .as_ref()
        .and_then(|v| v.loaders.first())
        .and_then(|l| match l.as_str() {
            "fabric" => Some(super::dto::ModLoader::Fabric),
            "neoforge" | "forge" => Some(super::dto::ModLoader::NeoForge),
            _ => None,
        });

    Some(ModInfo {
        path: path.to_path_buf(),
        filename: stem,
        enabled,
        size,
        id: Some(project.slug),
        name: Some(project.title),
        version: version_number,
        description: Some(project.description),
        authors: None,
        license: project.license.map(|l| l.name),
        minecraft,
        loader,
        icon: None,
        dependencies: Vec::new(),
    })
}

#[cfg(not(feature = "modrinth"))]
async fn try_modrinth(_path: &Path) -> Option<ModInfo> {
    None
}
