use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub badges: u64,
    #[serde(default)]
    pub auth_providers: Vec<String>,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub has_password: bool,
    #[serde(default)]
    pub has_totp: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TeamMember {
    #[serde(default)]
    pub team_id: String,
    #[serde(default)]
    pub user: Option<UserRef>,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub ordering: Option<i64>,
    #[serde(default)]
    pub permissions: Option<u64>,
    #[serde(default)]
    pub accepted: bool,
}

use super::project::UserRef;
