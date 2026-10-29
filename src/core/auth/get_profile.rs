use serde_json::json;

use super::dto::{McLoginResponse, McProfile, McProfileResponse, XblResponse, XstsResponse};
use super::error::AuthError;

const XBL_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

pub async fn get_profile(ms_access_token: &str) -> Result<McProfile, AuthError> {
    let client = reqwest::Client::new();

    let xbl: XblResponse = client
        .post(XBL_URL)
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={ms_access_token}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let xuid = xbl
        .display_claims
        .xui
        .first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| AuthError::InvalidResponse("xbl: empty DisplayClaims.xui".to_string()))?;

    let xsts: XstsResponse = client
        .post(XSTS_URL)
        .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl.token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mc_response = client
        .post(MC_LOGIN_URL)
        .json(&json!({
            "identityToken": format!("XBL3.0 x={xuid};{}", xsts.token)
        }))
        .send()
        .await?;

    if !mc_response.status().is_success() {
        return Ok(McProfile {
            username: "no_license".to_string(),
            uuid: "no_license".to_string(),
            access_token: "no_license".to_string(),
            xuid,
            has_license: false,
        });
    }

    let mc_login: McLoginResponse = mc_response.json().await?;

    let profile_response = client
        .get(MC_PROFILE_URL)
        .bearer_auth(&mc_login.access_token)
        .send()
        .await?;

    if profile_response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(McProfile {
            username: "no_license".to_string(),
            uuid: "no_license".to_string(),
            access_token: mc_login.access_token,
            xuid,
            has_license: false,
        });
    }

    let profile: McProfileResponse = profile_response.error_for_status()?.json().await?;

    Ok(McProfile {
        username: profile.name,
        uuid: profile.id,
        access_token: mc_login.access_token,
        xuid,
        has_license: true,
    })
}
