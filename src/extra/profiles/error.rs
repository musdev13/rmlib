use std::fmt;

#[derive(Debug)]
pub enum ProfileError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(musutils::fs::config::ConfigError),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::Io(e) => write!(f, "io error: {e}"),
            ProfileError::Json(e) => write!(f, "json error: {e}"),
            ProfileError::Config(e) => write!(f, "config error: {e}"),
        }
    }
}

impl std::error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProfileError::Io(e) => Some(e),
            ProfileError::Json(e) => Some(e),
            ProfileError::Config(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for ProfileError {
    fn from(e: std::io::Error) -> Self {
        ProfileError::Io(e)
    }
}

impl From<serde_json::Error> for ProfileError {
    fn from(e: serde_json::Error) -> Self {
        ProfileError::Json(e)
    }
}

impl From<musutils::fs::config::ConfigError> for ProfileError {
    fn from(e: musutils::fs::config::ConfigError) -> Self {
        ProfileError::Config(e)
    }
}
