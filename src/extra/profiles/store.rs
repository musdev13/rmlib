use std::path::{Path, PathBuf};

use musutils::fs::config::ConfigError;

use super::dto::{Profile, ProfileKind};
use super::error::ProfileError;

pub(crate) fn default_auth_dir() -> Result<PathBuf, ConfigError> {
    let paths = crate::core::config::Paths::load()?;
    Ok(paths.auth_pathbuf())
}

fn resolve_auth_dir(auth_dir: Option<&Path>) -> Result<PathBuf, ProfileError> {
    match auth_dir {
        Some(p) => Ok(p.to_path_buf()),
        None => Ok(default_auth_dir()?),
    }
}

fn validate_username(username: &str) -> Result<(), ProfileError> {
    let bad = username.is_empty()
        || username == "."
        || username == ".."
        || username.contains('/')
        || username.contains('\\');
    if bad {
        return Err(ProfileError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid profile username: {username:?}"),
        )));
    }
    Ok(())
}

pub fn profiles_dir_at(
    auth_dir: Option<&Path>,
    kind: ProfileKind,
) -> Result<PathBuf, ProfileError> {
    Ok(resolve_auth_dir(auth_dir)?
        .join("profiles")
        .join(kind.as_dir()))
}

pub fn profile_path_at(
    auth_dir: Option<&Path>,
    kind: ProfileKind,
    username: &str,
) -> Result<PathBuf, ProfileError> {
    validate_username(username)?;
    Ok(profiles_dir_at(auth_dir, kind)?.join(format!("{username}.json")))
}

pub fn save_profile_at(
    auth_dir: Option<&Path>,
    kind: ProfileKind,
    profile: &Profile,
) -> Result<PathBuf, ProfileError> {
    validate_username(&profile.username)?;
    let path = profile_path_at(auth_dir, kind, &profile.username)?;
    let text = serde_json::to_string_pretty(profile)?;
    musutils::fs::write_atomic(&path, text)?;
    Ok(path)
}

pub fn load_profile_at(
    auth_dir: Option<&Path>,
    kind: ProfileKind,
    username: &str,
) -> Result<Option<Profile>, ProfileError> {
    let path = profile_path_at(auth_dir, kind, username)?;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&text)?))
}

pub fn delete_profile_at(
    auth_dir: Option<&Path>,
    kind: ProfileKind,
    username: &str,
) -> Result<(), ProfileError> {
    let path = profile_path_at(auth_dir, kind, username)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn list_profiles_at(
    auth_dir: Option<&Path>,
    kind: ProfileKind,
) -> Result<Vec<String>, ProfileError> {
    let dir = profiles_dir_at(auth_dir, kind)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            out.push(stem.to_string());
        }
    }
    out.sort();
    Ok(out)
}

pub fn save_profile(kind: ProfileKind, profile: &Profile) -> Result<PathBuf, ProfileError> {
    save_profile_at(None, kind, profile)
}

pub fn load_profile(kind: ProfileKind, username: &str) -> Result<Option<Profile>, ProfileError> {
    load_profile_at(None, kind, username)
}

pub fn delete_profile(kind: ProfileKind, username: &str) -> Result<(), ProfileError> {
    delete_profile_at(None, kind, username)
}

pub fn list_profiles(kind: ProfileKind) -> Result<Vec<String>, ProfileError> {
    list_profiles_at(None, kind)
}

pub fn profile_path(kind: ProfileKind, username: &str) -> Result<PathBuf, ProfileError> {
    profile_path_at(None, kind, username)
}
