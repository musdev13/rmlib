use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsTokens {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McProfile {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub xuid: String,
    pub has_license: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default)]
    pub interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenErrorResponse {
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct XblResponse {
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DisplayClaims {
    pub xui: Vec<Xui>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Xui {
    pub uhs: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct XstsResponse {
    #[serde(rename = "Token")]
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct McLoginResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct McProfileResponse {
    pub id: String,
    pub name: String,
}
