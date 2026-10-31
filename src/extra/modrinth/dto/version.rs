use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

impl VersionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VersionType::Release => "release",
            VersionType::Beta => "beta",
            VersionType::Alpha => "alpha",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Hashes {
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub sha512: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VersionFile {
    #[serde(default)]
    pub hashes: Hashes,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub file_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VersionDependency {
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub dependency_type: String,
}

impl VersionDependency {
    pub fn is_required(&self) -> bool {
        self.dependency_type == "required"
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Version {
    pub id: String,
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub author_id: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version_number: String,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub date_published: String,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub requested_status: Option<String>,
    #[serde(default)]
    pub files: Vec<VersionFile>,
    #[serde(default)]
    pub dependencies: Vec<VersionDependency>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
}

impl Version {
    pub fn primary_file(&self) -> Option<&VersionFile> {
        self.files
            .iter()
            .find(|f| f.primary)
            .or_else(|| self.files.first())
    }

    pub fn is_release(&self) -> bool {
        self.version_type == "release"
    }

    pub fn is_beta(&self) -> bool {
        self.version_type == "beta"
    }

    pub fn is_alpha(&self) -> bool {
        self.version_type == "alpha"
    }

    pub fn supports_game_version(&self, game_version: &str) -> bool {
        self.game_versions.iter().any(|v| v == game_version)
    }

    pub fn supports_loader(&self, loader: &str) -> bool {
        self.loaders.iter().any(|l| l == loader)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VersionFilters {
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<String>>,
    pub version_type: Option<VersionType>,
}
