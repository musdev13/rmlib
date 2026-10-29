use std::path::Path;

use super::dto::McProfile;
use super::error::AuthError;
use super::{get_profile, load_tokens_at, login_at, refresh_at};

pub async fn get_data_at(dir: Option<&Path>) -> Result<McProfile, AuthError> {
    let ms_token = match load_tokens_at(dir)? {
        Some(tokens) => match refresh_at(dir, &tokens.refresh_token).await {
            Ok(refreshed) => refreshed.access_token,
            Err(_) => login_at(dir).await?.access_token,
        },
        None => login_at(dir).await?.access_token,
    };

    get_profile(&ms_token).await
}

pub async fn get_data() -> Result<McProfile, AuthError> {
    get_data_at(None).await
}
