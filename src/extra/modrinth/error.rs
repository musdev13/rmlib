use std::fmt;

#[derive(Debug)]
pub enum ModrinthError {
    Http(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(musutils::fs::config::ConfigError),
    Status { code: u16, body: String },
    Api { code: String, description: String },
    InvalidResponse(String),
    NotFound(String),
    RateLimited { retry_after: Option<u64> },
    HashMismatch { expected: String, actual: String },
    UnsupportedContentType(String),
    NoCompatibleVersion(String),
    DependencyCycle(String),
    InvalidArgument(String),
}

impl fmt::Display for ModrinthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModrinthError::Http(e) => write!(f, "http error: {e}"),
            ModrinthError::Io(e) => write!(f, "io error: {e}"),
            ModrinthError::Json(e) => write!(f, "json error: {e}"),
            ModrinthError::Config(e) => write!(f, "config error: {e}"),
            ModrinthError::Status { code, body } => write!(f, "http status {code}: {body}"),
            ModrinthError::Api { code, description } => {
                write!(f, "api error {code}: {description}")
            }
            ModrinthError::InvalidResponse(msg) => write!(f, "invalid response: {msg}"),
            ModrinthError::NotFound(what) => write!(f, "not found: {what}"),
            ModrinthError::RateLimited { retry_after } => match retry_after {
                Some(s) => write!(f, "rate limited, retry after {s}s"),
                None => write!(f, "rate limited"),
            },
            ModrinthError::HashMismatch { expected, actual } => {
                write!(f, "hash mismatch: expected {expected}, got {actual}")
            }
            ModrinthError::UnsupportedContentType(t) => {
                write!(f, "unsupported content type: {t}")
            }
            ModrinthError::NoCompatibleVersion(msg) => {
                write!(f, "no compatible version: {msg}")
            }
            ModrinthError::DependencyCycle(msg) => write!(f, "dependency cycle: {msg}"),
            ModrinthError::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
        }
    }
}

impl std::error::Error for ModrinthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ModrinthError::Http(e) => Some(e),
            ModrinthError::Io(e) => Some(e),
            ModrinthError::Json(e) => Some(e),
            ModrinthError::Config(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ModrinthError {
    fn from(e: reqwest::Error) -> Self {
        ModrinthError::Http(e)
    }
}

impl From<std::io::Error> for ModrinthError {
    fn from(e: std::io::Error) -> Self {
        ModrinthError::Io(e)
    }
}

impl From<serde_json::Error> for ModrinthError {
    fn from(e: serde_json::Error) -> Self {
        ModrinthError::Json(e)
    }
}

impl From<musutils::fs::config::ConfigError> for ModrinthError {
    fn from(e: musutils::fs::config::ConfigError) -> Self {
        ModrinthError::Config(e)
    }
}
