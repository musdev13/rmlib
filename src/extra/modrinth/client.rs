use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::OnceLock;
use std::time::Duration;

use super::error::ModrinthError;

pub const API_BASE: &str = "https://api.modrinth.com/v2";

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub(crate) fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

pub(crate) async fn get_json<T: DeserializeOwned>(
    path: &str,
    query: &[(&str, String)],
) -> Result<T, ModrinthError> {
    let url = format!("{API_BASE}{path}");
    let mut req = client().get(&url);
    if !query.is_empty() {
        req = req.query(query);
    }
    let response = req.send().await?;
    parse_response(response).await
}

pub(crate) fn build_error(status: u16, body: &str) -> ModrinthError {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        let code = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !code.is_empty() || !description.is_empty() {
            return ModrinthError::Api { code, description };
        }
    }
    if status == 429 {
        return ModrinthError::RateLimited { retry_after: None };
    }
    if status == 404 {
        return ModrinthError::NotFound(body.to_string());
    }
    ModrinthError::Status {
        code: status,
        body: body.to_string(),
    }
}

pub(crate) async fn parse_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, ModrinthError> {
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(build_error(status.as_u16(), &text));
    }
    Ok(serde_json::from_str::<T>(&text)?)
}
