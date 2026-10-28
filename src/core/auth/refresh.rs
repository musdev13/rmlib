use super::dto::{MsTokens, TokenResponse};
use super::error::AuthError;
use super::store::save_tokens;

const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";

pub async fn refresh(refresh_token: &str) -> Result<MsTokens, AuthError> {
    let client = reqwest::Client::new();

    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", crate::CLIENT_ID),
            ("refresh_token", refresh_token),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(AuthError::Status {
            code: status.as_u16(),
            body: text,
        });
    }

    let tokens: TokenResponse = serde_json::from_str(&text)?;
    let tokens = MsTokens {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_in: tokens.expires_in,
    };
    save_tokens(&tokens)?;
    Ok(tokens)
}
