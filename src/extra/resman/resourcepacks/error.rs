use std::fmt;

#[derive(Debug)]
pub enum RpError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    Modrinth(String),
    InvalidArgument(String),
    NotFound(String),
}

impl fmt::Display for RpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpError::Io(e) => write!(f, "io error: {e}"),
            RpError::Zip(e) => write!(f, "zip error: {e}"),
            RpError::Json(e) => write!(f, "json error: {e}"),
            RpError::Modrinth(s) => write!(f, "modrinth error: {s}"),
            RpError::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
            RpError::NotFound(s) => write!(f, "not found: {s}"),
        }
    }
}

impl std::error::Error for RpError {}

impl From<std::io::Error> for RpError {
    fn from(e: std::io::Error) -> Self {
        RpError::Io(e)
    }
}

impl From<zip::result::ZipError> for RpError {
    fn from(e: zip::result::ZipError) -> Self {
        RpError::Zip(e)
    }
}

impl From<serde_json::Error> for RpError {
    fn from(e: serde_json::Error) -> Self {
        RpError::Json(e)
    }
}
