use std::path::PathBuf;
use std::sync::LazyLock;

use serde_json::{Value, json};

use musutils::fs::config::{Config, ConfigError};

const DEFAULT_AUTH_PATH: &str = "~/rml/auth";

pub static DEFAULT_PATHS_CONFIG: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "versions_path": "~/rml/versions",
        "assets_path": "~/rml/assets",
        "libs_path": "~/rml/libs",
        "auth_path": DEFAULT_AUTH_PATH
    })
});

pub(crate) const APP_NAME: &str = "rml";
pub(crate) const FILE_NAME: &str = "paths.json";

const REQUIRED_KEYS: [&str; 4] = ["versions_path", "libs_path", "assets_path", "auth_path"];

#[derive(Debug, Clone)]
pub struct Paths {
    pub versions: String,
    pub libs: String,
    pub assets: String,
    pub auth: String,
}

impl Paths {
    pub fn load() -> Result<Self, ConfigError> {
        let default =
            serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).map_err(ConfigError::Parse)?;
        let cfg = Config::load_or_create(APP_NAME, FILE_NAME, &default)?;
        let auth = match cfg.get_as::<String>("auth_path") {
            Ok(value) => value,
            Err(ConfigError::KeyNotFound(_)) => DEFAULT_AUTH_PATH.to_string(),
            Err(e) => return Err(e),
        };
        Ok(Self {
            versions: cfg.get_as::<String>("versions_path")?,
            libs: cfg.get_as::<String>("libs_path")?,
            assets: cfg.get_as::<String>("assets_path")?,
            auth,
        })
    }

    pub fn init(force: bool) -> Result<(), ConfigError> {
        let default =
            serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).map_err(ConfigError::Parse)?;
        if force {
            musutils::fs::config::rewrite(APP_NAME, FILE_NAME, &default)?;
            return Ok(());
        }
        let mut cfg = Config::load_or_create(APP_NAME, FILE_NAME, &default)?;
        let mut changed = false;
        for key in REQUIRED_KEYS {
            match cfg.get_as::<String>(key) {
                Ok(_) => {}
                Err(ConfigError::KeyNotFound(_)) => {
                    let value = DEFAULT_PATHS_CONFIG
                        .get(key)
                        .cloned()
                        .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;
                    cfg.set(key, value)?;
                    changed = true;
                }
                Err(e) => return Err(e),
            }
        }
        if changed {
            cfg.save()?;
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

    pub fn auth_pathbuf(&self) -> PathBuf {
        musutils::fs::expand(&self.auth)
    }
}
