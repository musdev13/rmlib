use std::fmt;

#[derive(Debug)]
pub enum ModError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    Toml(toml::de::Error),
    Modrinth(String),
    InvalidArgument(String),
    NotFound(String),
}

impl fmt::Display for ModError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModError::Io(e) => write!(f, "io error: {e}"),
            ModError::Zip(e) => write!(f, "zip error: {e}"),
            ModError::Json(e) => write!(f, "json error: {e}"),
            ModError::Toml(e) => write!(f, "toml error: {e}"),
            ModError::Modrinth(s) => write!(f, "modrinth error: {s}"),
            ModError::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
            ModError::NotFound(s) => write!(f, "not found: {s}"),
        }
    }
}

impl std::error::Error for ModError {}

impl From<std::io::Error> for ModError {
    fn from(e: std::io::Error) -> Self {
        ModError::Io(e)
    }
}

impl From<zip::result::ZipError> for ModError {
    fn from(e: zip::result::ZipError) -> Self {
        ModError::Zip(e)
    }
}

impl From<serde_json::Error> for ModError {
    fn from(e: serde_json::Error) -> Self {
        ModError::Json(e)
    }
}

impl From<toml::de::Error> for ModError {
    fn from(e: toml::de::Error) -> Self {
        ModError::Toml(e)
    }
}
