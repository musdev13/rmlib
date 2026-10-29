use musutils::fs::config::{Config, ConfigError};

use super::paths::{APP_NAME, FILE_NAME};

pub fn check_config() -> Result<(), ConfigError> {
    let cfg = Config::load(APP_NAME, FILE_NAME)?;
    cfg.get_as::<String>("versions_path")?;
    cfg.get_as::<String>("libs_path")?;
    cfg.get_as::<String>("assets_path")?;
    cfg.get_as::<String>("auth_path")?;
    Ok(())
}
