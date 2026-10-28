#![cfg(feature = "core")]

mod common;

use std::fs;

use common::{HomeGuard, TempDir};
use rmlib::core::auth::{McProfile, MsTokens, clear_tokens, load_tokens, save_tokens, tokens_file};

fn sample_tokens() -> MsTokens {
    MsTokens {
        access_token: "access".to_string(),
        refresh_token: "refresh".to_string(),
        expires_in: Some(3600),
    }
}

fn ms_json(home: &TempDir) -> std::path::PathBuf {
    home.path().join("rml").join("auth").join("ms.json")
}

#[test]
fn tokens_file_uses_auth_path() {
    let home = TempDir::new("auth_tokens_file");
    let _guard = HomeGuard::new(home.path());

    let path = tokens_file().unwrap();
    assert_eq!(path, ms_json(&home));
}

#[test]
fn save_creates_file_in_auth_path() {
    let home = TempDir::new("auth_save");
    let _guard = HomeGuard::new(home.path());

    save_tokens(&sample_tokens()).unwrap();

    let f = ms_json(&home);
    assert!(f.exists(), "ms.json should be created");
    let text = fs::read_to_string(&f).unwrap();
    assert!(text.contains("\"access_token\": \"access\""));
    assert!(text.contains("\"refresh_token\": \"refresh\""));
}

#[test]
fn load_roundtrip() {
    let home = TempDir::new("auth_roundtrip");
    let _guard = HomeGuard::new(home.path());

    let tokens = sample_tokens();
    save_tokens(&tokens).unwrap();
    let loaded = load_tokens().unwrap().expect("tokens should load");
    assert_eq!(loaded, tokens);
}

#[test]
fn load_missing_returns_none() {
    let home = TempDir::new("auth_missing");
    let _guard = HomeGuard::new(home.path());

    assert!(load_tokens().unwrap().is_none());
}

#[test]
fn load_corrupt_returns_none() {
    let home = TempDir::new("auth_corrupt");
    let _guard = HomeGuard::new(home.path());

    let f = ms_json(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(&f, b"not json").unwrap();

    assert!(load_tokens().unwrap().is_none());
}

#[test]
fn clear_removes_file() {
    let home = TempDir::new("auth_clear");
    let _guard = HomeGuard::new(home.path());

    save_tokens(&sample_tokens()).unwrap();
    assert!(ms_json(&home).exists());

    clear_tokens().unwrap();
    assert!(!ms_json(&home).exists());
}

#[test]
fn clear_missing_is_ok() {
    let home = TempDir::new("auth_clear_missing");
    let _guard = HomeGuard::new(home.path());

    clear_tokens().unwrap();
}

#[test]
fn ms_tokens_serde_omits_missing_expires() {
    let tokens: MsTokens =
        serde_json::from_str(r#"{"access_token":"a","refresh_token":"r"}"#).unwrap();
    assert_eq!(tokens.access_token, "a");
    assert_eq!(tokens.refresh_token, "r");
    assert!(tokens.expires_in.is_none());
}

#[test]
fn mc_profile_serializes_all_fields() {
    let profile = McProfile {
        username: "Steve".to_string(),
        uuid: "uuid".to_string(),
        access_token: "token".to_string(),
        xuid: "xuid".to_string(),
        has_license: true,
    };
    let value = serde_json::to_value(&profile).unwrap();
    assert_eq!(value["username"], "Steve");
    assert_eq!(value["uuid"], "uuid");
    assert_eq!(value["access_token"], "token");
    assert_eq!(value["xuid"], "xuid");
    assert_eq!(value["has_license"], true);
}

#[test]
fn no_license_profile_shape() {
    let profile = McProfile {
        username: "no_license".to_string(),
        uuid: "no_license".to_string(),
        access_token: "no_license".to_string(),
        xuid: "real_xuid".to_string(),
        has_license: false,
    };
    let value = serde_json::to_value(&profile).unwrap();
    assert_eq!(value["username"], "no_license");
    assert_eq!(value["has_license"], false);
    assert_eq!(value["xuid"], "real_xuid");
}
