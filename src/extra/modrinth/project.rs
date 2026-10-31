use super::client::get_json;
use super::dto::{Dependency, Project, ProjectMember, VersionFilters};
use super::error::ModrinthError;

pub async fn get_project(id_or_slug: &str) -> Result<Project, ModrinthError> {
    if id_or_slug.is_empty() {
        return Err(ModrinthError::InvalidArgument(
            "project id/slug is empty".to_string(),
        ));
    }
    get_json(&format!("/project/{id_or_slug}"), &[]).await
}

pub async fn get_projects(ids: &[String]) -> Result<Vec<Project>, ModrinthError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let ids_json = serde_json::to_string(ids)?;
    get_json("/projects", &[("ids", ids_json)]).await
}

pub async fn get_project_by_hash(hash: &str) -> Result<Project, ModrinthError> {
    let version = super::version::get_version_by_hash(hash).await?;
    get_project(&version.project_id).await
}

pub async fn get_project_dependencies(project_id: &str) -> Result<Vec<Dependency>, ModrinthError> {
    get_project_dependencies_filtered(project_id, &VersionFilters::default()).await
}

pub async fn get_project_dependencies_filtered(
    project_id: &str,
    filters: &VersionFilters,
) -> Result<Vec<Dependency>, ModrinthError> {
    let versions = super::version::get_project_versions(project_id, filters).await?;
    let mut out: Vec<Dependency> = Vec::new();
    for v in versions {
        for d in v.dependencies {
            let dep = Dependency {
                version_id: d.version_id,
                project_id: d.project_id,
                file_name: d.file_name,
                dependency_type: d.dependency_type,
            };
            let key = (
                dep.project_id.clone(),
                dep.version_id.clone(),
                dep.dependency_type.clone(),
            );
            let exists = out.iter().any(|e| {
                (
                    e.project_id.clone(),
                    e.version_id.clone(),
                    e.dependency_type.clone(),
                ) == key
            });
            if !exists {
                out.push(dep);
            }
        }
    }
    Ok(out)
}

pub async fn get_project_members(project_id: &str) -> Result<Vec<ProjectMember>, ModrinthError> {
    get_json(&format!("/project/{project_id}/members"), &[]).await
}
