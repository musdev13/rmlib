use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct License {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GalleryItem {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub slug: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub client_side: String,
    #[serde(default)]
    pub server_side: String,
    #[serde(default)]
    pub project_type: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub followers: u64,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub issues_url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub wiki_url: Option<String>,
    #[serde(default)]
    pub discord_url: Option<String>,
    #[serde(default)]
    pub donation_urls: Vec<DonationUrl>,
    #[serde(default)]
    pub license: Option<License>,
    #[serde(default)]
    pub versions: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub gallery: Vec<GalleryItem>,
    #[serde(default)]
    pub team: String,
    #[serde(default)]
    pub published: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub requested_status: Option<String>,
    #[serde(default)]
    pub moderator_message: Option<ModeratorMessage>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DonationUrl {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub platform: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ModeratorMessage {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Dependency {
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub dependency_type: String,
}

impl Dependency {
    pub fn is_required(&self) -> bool {
        self.dependency_type == "required"
    }

    pub fn is_optional(&self) -> bool {
        self.dependency_type == "optional"
    }

    pub fn is_incompatible(&self) -> bool {
        self.dependency_type == "incompatible"
    }

    pub fn is_embedded(&self) -> bool {
        self.dependency_type == "embedded"
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProjectMember {
    #[serde(default)]
    pub team_id: String,
    #[serde(default)]
    pub user: Option<UserRef>,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub ordering: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UserRef {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}
