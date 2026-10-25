use std::path::PathBuf;
use std::sync::LazyLock;

use serde_json::{Value, json};

use musutils::fs::config::{Config, ConfigError};

pub static DEFAULT_PATHS_CONFIG: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "versions_path": "~/rml/versions",
        "assets_path": "~/rml/assets",
        "libs_path": "~/rml/libs"
    })
});

const APP_NAME: &str = "rml";
const FILE_NAME: &str = "paths.json";

#[derive(Debug, Clone)]
pub struct Paths {
    pub versions: String,
    pub libs: String,
    pub assets: String,
}

impl Paths {
    pub fn load() -> Result<Self, ConfigError> {
        let default =
            serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).map_err(ConfigError::Parse)?;
        let cfg = Config::load_or_create(APP_NAME, FILE_NAME, &default)?;
        Ok(Self {
            versions: cfg.get_as::<String>("versions_path")?,
            libs: cfg.get_as::<String>("libs_path")?,
            assets: cfg.get_as::<String>("assets_path")?,
        })
    }

    pub fn init(force: bool) -> Result<(), ConfigError> {
        let default =
            serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).map_err(ConfigError::Parse)?;
        if force {
            musutils::fs::config::rewrite(APP_NAME, FILE_NAME, &default)?;
        } else {
            Config::load_or_create(APP_NAME, FILE_NAME, &default)?;
        }
        Ok(())
    }

    pub fn versions_pathbuf(&self) -> PathBuf {
        musutils::fs::expand(&self.versions)
    }

    pub fn libs_pathbuf(&self) -> PathBuf {
        musutils::fs::expand(&self.libs)
    }

    pub fn assets_pathbuf(&self) -> PathBuf {
        musutils::fs::expand(&self.assets)
    }
}
