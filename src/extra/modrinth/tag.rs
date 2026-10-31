use super::client::get_json;
use super::dto::{Category, GameVersion, Loader};
use super::error::ModrinthError;

pub async fn get_categories() -> Result<Vec<Category>, ModrinthError> {
    get_json("/tag/category", &[]).await
}

pub async fn get_loaders() -> Result<Vec<Loader>, ModrinthError> {
    get_json("/tag/loader", &[]).await
}

pub async fn get_game_versions() -> Result<Vec<GameVersion>, ModrinthError> {
    get_json("/tag/game_version", &[]).await
}

pub async fn get_project_types() -> Result<Vec<String>, ModrinthError> {
    get_json("/tag/project_type", &[]).await
}
