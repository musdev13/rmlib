use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AuthRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    #[serde(rename = "clientToken")]
    pub client_token: &'a str,
    #[serde(rename = "requestUser")]
    pub request_user: bool,
}

#[derive(Deserialize, Debug)]
pub struct ProfileResponse {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct AuthApiResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "clientToken")]
    pub client_token: String,
    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<ProfileResponse>,
}

#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub access_token: String,
    pub client_token: String,
    pub profile_id: String,
    pub profile_name: String,
}
