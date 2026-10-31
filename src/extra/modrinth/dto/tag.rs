use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Category {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub project_type: String,
    #[serde(default)]
    pub header: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Loader {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub supported_project_types: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameVersion {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub major: bool,
}
