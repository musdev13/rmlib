use std::path::Path;
use std::time::Duration;

use musutils::types::Status;

use super::dto::{DeviceCodeResponse, MsTokens, TokenErrorResponse, TokenResponse};
use super::error::AuthError;
use super::store::save_tokens_at;

const DEVICE_CODE_URL: &str = "https://login.live.com/oauth20_connect.srf";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const SCOPE: &str = "XboxLive.signin offline_access";

pub async fn login_at(dir: Option<&Path>) -> Result<MsTokens, AuthError> {
    let client = reqwest::Client::new();

    let device: DeviceCodeResponse = client
        .post(DEVICE_CODE_URL)
        .form(&[
            ("client_id", crate::CLIENT_ID),
            ("scope", SCOPE),
            ("response_type", "device_code"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    println!(
        "{}: open {} and enter code {}",
        Status::Note.as_colored_str(),
        device.verification_uri,
        device.user_code
    );

    let mut interval = device.interval.unwrap_or(5).max(1);

    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        let response = client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", crate::CLIENT_ID),
                ("device_code", &device.device_code),
            ])
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            let tokens: TokenResponse = serde_json::from_str(&text)?;
            let tokens = MsTokens {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
                expires_in: tokens.expires_in,
            };
            save_tokens_at(dir, &tokens)?;
            return Ok(tokens);
        }

        let err: TokenErrorResponse =
            serde_json::from_str(&text).map_err(|_| AuthError::Status {
                code: status.as_u16(),
                body: text.clone(),
            })?;

        match err.error.as_str() {
            "authorization_pending" => continue,
            "slow_down" => {
                interval += 5;
                continue;
            }
            "expired_token" => return Err(AuthError::DeviceCodeExpired),
            _ => {
                return Err(AuthError::DeviceCode {
                    error: err.error,
                    description: err.error_description,
                });
            }
        }
    }
}

pub async fn login() -> Result<MsTokens, AuthError> {
    login_at(None).await
}
