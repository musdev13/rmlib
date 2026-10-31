use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::ProfileError;

pub const DEFAULT_INSTANCE_FILE: &str = "rml.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ely: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub betafix: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jvm_args: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_args: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versions: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libs: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neoforge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_path: Option<PathBuf>,
}

impl InstanceConfig {
    pub fn merge_missing_from(&mut self, other: &InstanceConfig) {
        macro_rules! fill {
            ($($field:ident),* $(,)?) => {
                $(
                    if self.$field.is_none() {
                        self.$field = other.$field.clone();
                    }
                )*
            };
        }
        fill!(
            version_id, ram, username, ely, betafix, profile, jvm_args, game_args, versions,
            assets, libs, game_path, spawn_path, java, fabric, neoforge, auth_path,
        );
    }
}

pub fn instance_default_path(game_path: &Path) -> PathBuf {
    game_path.join(DEFAULT_INSTANCE_FILE)
}

pub fn save_instance(path: &Path, cfg: &InstanceConfig) -> Result<(), ProfileError> {
    let text = serde_json::to_string_pretty(cfg)?;
    musutils::fs::write_atomic(path, text)?;
    Ok(())
}

pub fn load_instance(path: &Path) -> Result<InstanceConfig, ProfileError> {
    if !path.exists() {
        return Err(ProfileError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("instance file not found: {}", path.display()),
        )));
    }
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}
