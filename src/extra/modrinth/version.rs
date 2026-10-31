use super::client::get_json;
use super::dto::{Version, VersionFilters, VersionType};
use super::error::ModrinthError;

pub async fn get_version(version_id: &str) -> Result<Version, ModrinthError> {
    if version_id.is_empty() {
        return Err(ModrinthError::InvalidArgument(
            "version id is empty".to_string(),
        ));
    }
    get_json(&format!("/version/{version_id}"), &[]).await
}

pub async fn get_version_by_hash(hash: &str) -> Result<Version, ModrinthError> {
    if hash.is_empty() {
        return Err(ModrinthError::InvalidArgument("hash is empty".to_string()));
    }
    get_json(&format!("/version_file/{hash}"), &[]).await
}

pub async fn get_version_from_hash(hash: &str) -> Result<Version, ModrinthError> {
    let mut versions: Vec<Version> = get_json(
        "/version_files",
        &[
            ("hashes", format!("[\"{hash}\"]")),
            ("algorithm", "sha1".to_string()),
        ],
    )
    .await?;
    if versions.is_empty() {
        return Err(ModrinthError::NotFound(format!(
            "no version for hash {hash}"
        )));
    }
    Ok(versions.remove(0))
}

pub async fn get_project_versions(
    project_id: &str,
    filters: &VersionFilters,
) -> Result<Vec<Version>, ModrinthError> {
    if project_id.is_empty() {
        return Err(ModrinthError::InvalidArgument(
            "project id is empty".to_string(),
        ));
    }
    let mut q: Vec<(&str, String)> = Vec::new();
    if let Some(gv) = &filters.game_versions {
        let s = serde_json::to_string(gv)?;
        q.push(("game_versions", s));
    }
    if let Some(loaders) = &filters.loaders {
        let s = serde_json::to_string(loaders)?;
        q.push(("loaders", s));
    }
    get_json(&format!("/project/{project_id}/version"), &q).await
}

pub fn filter_versions(versions: &[Version], filters: &VersionFilters) -> Vec<Version> {
    versions
        .iter()
        .filter(|v| match filters.version_type {
            Some(VersionType::Release) => v.is_release(),
            Some(VersionType::Beta) => v.is_beta(),
            Some(VersionType::Alpha) => v.is_alpha(),
            None => true,
        })
        .filter(|v| match &filters.game_versions {
            Some(gvs) => gvs.iter().any(|g| v.supports_game_version(g)),
            None => true,
        })
        .filter(|v| match &filters.loaders {
            Some(ls) => ls.iter().any(|l| v.supports_loader(l)),
            None => true,
        })
        .cloned()
        .collect()
}

pub fn select_compatible_version<'a>(
    versions: &'a [Version],
    game_version: Option<&str>,
    loader: Option<&str>,
    version_type: Option<VersionType>,
) -> Option<&'a Version> {
    versions.iter().find(|v| {
        game_version
            .map(|g| v.supports_game_version(g))
            .unwrap_or(true)
            && loader.map(|l| v.supports_loader(l)).unwrap_or(true)
            && match version_type {
                Some(VersionType::Release) => v.is_release(),
                Some(VersionType::Beta) => v.is_beta(),
                Some(VersionType::Alpha) => v.is_alpha(),
                None => true,
            }
    })
}
