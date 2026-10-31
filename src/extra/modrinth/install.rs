use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::client::client;
use super::dto::{
    InstallParams, InstallResult, InstallSource, InstalledFile, ProjectType, SkipReason,
    SkippedDependency, Version, VersionFile, VersionFilters,
};
use super::error::ModrinthError;
use super::hash;
use super::project;
use super::version;

pub async fn install_project(
    id_or_slug: &str,
    params: &InstallParams,
) -> Result<InstallResult, ModrinthError> {
    let proj = project::get_project(id_or_slug).await?;
    let project_type = parse_project_type(&proj.project_type);
    let filters = VersionFilters {
        game_versions: params.minecraft_version.clone().map(|v| vec![v]),
        loaders: params.loader.clone().map(|l| vec![l]),
        version_type: params.version_type,
    };
    let versions = version::get_project_versions(&proj.id, &filters).await?;
    let selected = version::select_compatible_version(
        &versions,
        params.minecraft_version.as_deref(),
        params.loader.as_deref(),
        params.version_type,
    )
    .ok_or_else(|| {
        ModrinthError::NoCompatibleVersion(format!(
            "project {} for mc={:?} loader={:?}",
            proj.slug, params.minecraft_version, params.loader
        ))
    })?;
    let selected = selected.clone();

    let target_dir = resolve_install_dir(project_type, params)?;

    let installed = install_version_files(&selected, &target_dir, params).await?;
    let mut all_installed = installed;
    let mut skipped: Vec<SkippedDependency> = Vec::new();

    if params.install_required_dependencies {
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(proj.id.clone());
        collect_dependencies(
            &selected,
            project_type,
            params,
            &mut visited,
            &mut all_installed,
            &mut skipped,
        )
        .await?;
    }

    Ok(InstallResult {
        source: InstallSource::Project,
        project_type: Some(project_type),
        installed: all_installed,
        skipped,
    })
}

pub async fn install_version(
    version_id: &str,
    params: &InstallParams,
) -> Result<InstallResult, ModrinthError> {
    let ver = version::get_version(version_id).await?;
    let proj = project::get_project(&ver.project_id).await?;
    let project_type = parse_project_type(&proj.project_type);
    let target_dir = resolve_install_dir(project_type, params)?;
    let installed = install_version_files(&ver, &target_dir, params).await?;
    Ok(InstallResult {
        source: InstallSource::Version,
        project_type: Some(project_type),
        installed,
        skipped: Vec::new(),
    })
}

pub async fn install_from_hash(
    hash: &str,
    params: &InstallParams,
) -> Result<InstallResult, ModrinthError> {
    let ver = version::get_version_by_hash(hash).await?;
    let proj = project::get_project(&ver.project_id).await?;
    let project_type = parse_project_type(&proj.project_type);
    let target_dir = resolve_install_dir(project_type, params)?;
    let installed = install_version_files(&ver, &target_dir, params).await?;
    Ok(InstallResult {
        source: InstallSource::Hash,
        project_type: Some(project_type),
        installed,
        skipped: Vec::new(),
    })
}

pub async fn install_datapack(
    id_or_slug: &str,
    world_name: &str,
    game_path: &Path,
    params: &InstallParams,
) -> Result<InstallResult, ModrinthError> {
    validate_world_name(world_name)?;
    let world_dir = game_path.join("saves").join(world_name);
    if !world_dir.exists() {
        return Err(ModrinthError::InvalidArgument(format!(
            "world directory does not exist: {}",
            world_dir.display()
        )));
    }
    let target = world_dir.join("datapacks");

    let proj = project::get_project(id_or_slug).await?;
    let project_type = parse_project_type(&proj.project_type);
    if project_type != ProjectType::Datapack {
        return Err(ModrinthError::UnsupportedContentType(
            proj.project_type.clone(),
        ));
    }
    let filters = VersionFilters {
        game_versions: params.minecraft_version.clone().map(|v| vec![v]),
        loaders: None,
        version_type: params.version_type,
    };
    let versions = version::get_project_versions(&proj.id, &filters).await?;
    let selected = version::select_compatible_version(
        &versions,
        params.minecraft_version.as_deref(),
        None,
        params.version_type,
    )
    .ok_or_else(|| {
        ModrinthError::NoCompatibleVersion(format!("datapack {} for world {world_name}", proj.slug))
    })?
    .clone();
    let installed = install_version_files(&selected, &target, params).await?;
    Ok(InstallResult {
        source: InstallSource::Datapack,
        project_type: Some(project_type),
        installed,
        skipped: Vec::new(),
    })
}

pub fn resolve_install_dir(
    project_type: ProjectType,
    params: &InstallParams,
) -> Result<PathBuf, ModrinthError> {
    if let Some(dir) = &params.install_dir {
        return Ok(dir.clone());
    }
    let Some(game_path) = &params.game_path else {
        return Err(ModrinthError::InvalidArgument(
            "either install_dir or game_path is required".to_string(),
        ));
    };
    match project_type {
        ProjectType::Mod => Ok(game_path.join("mods")),
        ProjectType::Resourcepack => Ok(game_path.join("resourcepacks")),
        ProjectType::Shader => Ok(game_path.join("shaderpacks")),
        ProjectType::Datapack => Err(ModrinthError::UnsupportedContentType(
            "datapack requires a world; use install_datapack".to_string(),
        )),
        ProjectType::Modpack | ProjectType::World | ProjectType::Plugin => {
            Err(ModrinthError::UnsupportedContentType(format!(
                "{} requires an explicit install_dir",
                project_type.as_str()
            )))
        }
    }
}

pub fn parse_project_type(s: &str) -> ProjectType {
    match s {
        "mod" => ProjectType::Mod,
        "modpack" => ProjectType::Modpack,
        "resourcepack" => ProjectType::Resourcepack,
        "shader" => ProjectType::Shader,
        "datapack" => ProjectType::Datapack,
        "world" => ProjectType::World,
        "plugin" => ProjectType::Plugin,
        _ => ProjectType::Mod,
    }
}

pub fn validate_world_name(name: &str) -> Result<(), ModrinthError> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err(ModrinthError::InvalidArgument(format!(
            "invalid world name: {name:?}"
        )));
    }
    Ok(())
}

pub(crate) async fn install_version_files(
    ver: &Version,
    target_dir: &Path,
    params: &InstallParams,
) -> Result<Vec<InstalledFile>, ModrinthError> {
    let file = ver
        .primary_file()
        .ok_or_else(|| ModrinthError::InvalidResponse("version has no files".to_string()))?;
    let installed = download_file(ver, file, target_dir, params).await?;
    Ok(vec![installed])
}

async fn download_file(
    ver: &Version,
    file: &VersionFile,
    target_dir: &Path,
    params: &InstallParams,
) -> Result<InstalledFile, ModrinthError> {
    if file.url.is_empty() || file.filename.is_empty() {
        return Err(ModrinthError::InvalidResponse(
            "version file missing url or filename".to_string(),
        ));
    }
    std::fs::create_dir_all(target_dir)?;

    let candidate = target_dir.join(&file.filename);
    if !params.overwrite
        && candidate.exists()
        && let Some(expected) = &file.hashes.sha1
        && let Ok(actual) = hash::sha1_file(&candidate)
        && actual.eq_ignore_ascii_case(expected)
    {
        let bytes = std::fs::metadata(&candidate)
            .map(|m| m.len())
            .unwrap_or(file.size);
        return Ok(InstalledFile {
            path: candidate,
            filename: file.filename.clone(),
            bytes,
            sha1: file.hashes.sha1.clone(),
            sha512: file.hashes.sha512.clone(),
            project_id: Some(ver.project_id.clone()),
            version_id: Some(ver.id.clone()),
        });
    }

    let (final_path, actual_filename) =
        resolve_conflict(&candidate, params.overwrite, file.hashes.sha1.as_deref())?;

    let response = client().get(&file.url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(ModrinthError::Status {
            code: status.as_u16(),
            body: format!("download failed for {}", file.url),
        });
    }
    let bytes = response.bytes().await?;

    if let Some(expected) = &file.hashes.sha1 {
        let actual = hash::sha1_bytes(&bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(ModrinthError::HashMismatch {
                expected: expected.clone(),
                actual,
            });
        }
    }
    if let Some(expected) = &file.hashes.sha512 {
        let actual = hash::sha512_bytes(&bytes);
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(ModrinthError::HashMismatch {
                expected: expected.clone(),
                actual,
            });
        }
    }

    musutils::fs::write_atomic(&final_path, &bytes)?;

    Ok(InstalledFile {
        path: final_path,
        filename: actual_filename,
        bytes: bytes.len() as u64,
        sha1: file.hashes.sha1.clone(),
        sha512: file.hashes.sha512.clone(),
        project_id: Some(ver.project_id.clone()),
        version_id: Some(ver.id.clone()),
    })
}

fn resolve_conflict(
    path: &Path,
    overwrite: bool,
    sha1: Option<&str>,
) -> Result<(PathBuf, String), ModrinthError> {
    if !path.exists() {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        return Ok((path.to_path_buf(), name));
    }
    if overwrite {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        return Ok((path.to_path_buf(), name));
    }
    if let Some(expected) = sha1
        && let Ok(actual) = hash::sha1_file(path)
        && actual.eq_ignore_ascii_case(expected)
    {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        return Ok((path.to_path_buf(), name));
    }

    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    let ext = path
        .extension()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    for i in 1..1000 {
        let candidate = if ext.is_empty() {
            path.with_file_name(format!("{stem} ({i})"))
        } else {
            path.with_file_name(format!("{stem} ({i}).{ext}"))
        };
        if !candidate.exists() {
            let name = candidate
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            return Ok((candidate, name));
        }
    }
    Err(ModrinthError::InvalidArgument(format!(
        "could not resolve filename conflict for {}",
        path.display()
    )))
}

async fn collect_dependencies(
    ver: &Version,
    parent_type: ProjectType,
    params: &InstallParams,
    visited: &mut HashSet<String>,
    installed: &mut Vec<InstalledFile>,
    skipped: &mut Vec<SkippedDependency>,
) -> Result<(), ModrinthError> {
    for dep in &ver.dependencies {
        if !dep.is_required() {
            continue;
        }
        let Some(pid) = &dep.project_id else {
            continue;
        };
        if !visited.insert(pid.clone()) {
            continue;
        }
        let dep_project = match project::get_project(pid).await {
            Ok(p) => p,
            Err(_) => {
                skipped.push(SkippedDependency {
                    project_id: pid.clone(),
                    reason: SkipReason::ProjectUnresolved,
                });
                continue;
            }
        };
        let dep_type = parse_project_type(&dep_project.project_type);
        if dep_type != parent_type && !is_compatible_install(dep_type) {
            skipped.push(SkippedDependency {
                project_id: pid.clone(),
                reason: SkipReason::UnsupportedContentType,
            });
            continue;
        }
        let filters = VersionFilters {
            game_versions: params.minecraft_version.clone().map(|v| vec![v]),
            loaders: params.loader.clone().map(|l| vec![l]),
            version_type: params.version_type,
        };
        let versions = match version::get_project_versions(pid, &filters).await {
            Ok(v) => v,
            Err(_) => {
                skipped.push(SkippedDependency {
                    project_id: pid.clone(),
                    reason: SkipReason::ProjectUnresolved,
                });
                continue;
            }
        };
        let Some(selected) = version::select_compatible_version(
            &versions,
            params.minecraft_version.as_deref(),
            params.loader.as_deref(),
            params.version_type,
        ) else {
            skipped.push(SkippedDependency {
                project_id: pid.clone(),
                reason: SkipReason::NoCompatibleVersion,
            });
            continue;
        };
        let selected = selected.clone();
        let target_dir = match resolve_install_dir(dep_type, params) {
            Ok(d) => d,
            Err(_) => {
                skipped.push(SkippedDependency {
                    project_id: pid.clone(),
                    reason: SkipReason::UnsupportedContentType,
                });
                continue;
            }
        };
        let files = install_version_files(&selected, &target_dir, params).await?;
        installed.extend(files);
        Box::pin(collect_dependencies(
            &selected, dep_type, params, visited, installed, skipped,
        ))
        .await?;
    }
    Ok(())
}

fn is_compatible_install(t: ProjectType) -> bool {
    matches!(
        t,
        ProjectType::Mod | ProjectType::Resourcepack | ProjectType::Shader
    )
}
