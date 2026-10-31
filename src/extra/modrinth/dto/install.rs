use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::search::ProjectType;
use super::version::VersionType;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InstallParams {
    #[serde(default)]
    pub game_path: Option<PathBuf>,
    #[serde(default)]
    pub install_dir: Option<PathBuf>,
    #[serde(default)]
    pub minecraft_version: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub version_type: Option<VersionType>,
    #[serde(default)]
    pub install_required_dependencies: bool,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledFile {
    pub path: PathBuf,
    pub filename: String,
    pub bytes: u64,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallSource {
    Project,
    Version,
    Hash,
    Datapack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedDependency {
    pub project_id: String,
    pub reason: SkipReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    ProjectUnresolved,
    NoCompatibleVersion,
    UnsupportedContentType,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SkipReason::ProjectUnresolved => "project_unresolved",
            SkipReason::NoCompatibleVersion => "no_compatible_version",
            SkipReason::UnsupportedContentType => "unsupported_content_type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallResult {
    pub source: InstallSource,
    pub project_type: Option<ProjectType>,
    pub installed: Vec<InstalledFile>,
    #[serde(default)]
    pub skipped: Vec<SkippedDependency>,
}
