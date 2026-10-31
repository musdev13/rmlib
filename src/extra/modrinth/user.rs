use super::client::get_json;
use super::dto::{Project, TeamMember, User};
use super::error::ModrinthError;

pub async fn get_user(id_or_username: &str) -> Result<User, ModrinthError> {
    if id_or_username.is_empty() {
        return Err(ModrinthError::InvalidArgument(
            "user id/username is empty".to_string(),
        ));
    }
    get_json(&format!("/user/{id_or_username}"), &[]).await
}

pub async fn get_user_projects(user_id: &str) -> Result<Vec<Project>, ModrinthError> {
    if user_id.is_empty() {
        return Err(ModrinthError::InvalidArgument(
            "user id is empty".to_string(),
        ));
    }
    get_json(&format!("/user/{user_id}/projects"), &[]).await
}

pub async fn get_team_members(team_id: &str) -> Result<Vec<TeamMember>, ModrinthError> {
    if team_id.is_empty() {
        return Err(ModrinthError::InvalidArgument(
            "team id is empty".to_string(),
        ));
    }
    get_json(&format!("/team/{team_id}/members"), &[]).await
}
