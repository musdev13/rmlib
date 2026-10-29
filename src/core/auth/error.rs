use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Config(musutils::fs::config::ConfigError),
    Status {
        code: u16,
        body: String,
    },
    DeviceCode {
        error: String,
        description: Option<String>,
    },
    InvalidResponse(String),
    DeviceCodeExpired,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Io(e) => write!(f, "io error: {e}"),
            AuthError::Http(e) => write!(f, "http error: {e}"),
            AuthError::Json(e) => write!(f, "json error: {e}"),
            AuthError::Config(e) => write!(f, "config error: {e}"),
            AuthError::Status { code, body } => {
                write!(f, "http status {code}: {body}")
            }
            AuthError::DeviceCode { error, description } => match description {
                Some(d) => write!(f, "device code error: {error}: {d}"),
                None => write!(f, "device code error: {error}"),
            },
            AuthError::InvalidResponse(msg) => write!(f, "invalid response: {msg}"),
            AuthError::DeviceCodeExpired => write!(f, "device code expired"),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuthError::Io(e) => Some(e),
            AuthError::Http(e) => Some(e),
            AuthError::Json(e) => Some(e),
            AuthError::Config(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AuthError {
    fn from(e: std::io::Error) -> Self {
        AuthError::Io(e)
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::Http(e)
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(e: serde_json::Error) -> Self {
        AuthError::Json(e)
    }
}

impl From<musutils::fs::config::ConfigError> for AuthError {
    fn from(e: musutils::fs::config::ConfigError) -> Self {
        AuthError::Config(e)
    }
}
