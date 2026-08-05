use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AuthRequest<'a> {
    username: &'a str,
    password: &'a str,
    #[serde(rename = "clientToken")]
    client_token: &'a str,
    #[serde(rename = "requestUser")]
    request_user: bool,
}

#[derive(Deserialize, Debug)]
struct ProfileResponse {
    id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct AuthApiResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "selectedProfile")]
    selected_profile: Option<ProfileResponse>,
}

#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub access_token: String,
    pub client_token: String,
    pub profile_id: String,
    pub profile_name: String,
}

pub async fn login(login: String, passwd: String) -> Result<AuthResult, Box<dyn std::error::Error>> {
    let client = Client::new();

    let request_body = AuthRequest {
        username: &login,
        password: &passwd,
        client_token: "93301754-cd11-4a00-bd50-f69dffaf4b0b",
        request_user: true,
    };

    let response = client
        .post("https://authserver.ely.by/auth/authenticate")
        .json(&request_body)
        .send()
        .await?
        .json::<AuthApiResponse>()
        .await?;

    let profile = response
        .selected_profile
        .ok_or("Selected profile not found in response")?;

    Ok(AuthResult {
        access_token: response.access_token,
        client_token: response.client_token,
        profile_id: profile.id,
        profile_name: profile.name,
    })
}
