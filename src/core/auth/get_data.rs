use super::dto::McProfile;
use super::error::AuthError;
use super::{get_profile, load_tokens, login, refresh};

pub async fn get_data() -> Result<McProfile, AuthError> {
    let ms_token = match load_tokens()? {
        Some(tokens) => match refresh(&tokens.refresh_token).await {
            Ok(refreshed) => refreshed.access_token,
            Err(_) => login().await?.access_token,
        },
        None => login().await?.access_token,
    };

    get_profile(&ms_token).await
}
