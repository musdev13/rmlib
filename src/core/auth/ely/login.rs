use crate::core::auth::ely::dto::*;
use reqwest::Client;

pub async fn login(
    login: String,
    passwd: String,
) -> Result<AuthResult, Box<dyn std::error::Error>> {
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
