#![cfg(feature = "core")]

mod common;

use std::fs;
use std::path::PathBuf;

use common::{HomeGuard, TempDir};
use musutils::fs::config::ConfigError;
use rmlib::core::config::{
    DEFAULT_PATHS_CONFIG, Paths, check_config, get_assets_path, get_auth_path, get_libs_path,
    get_versions_path, init,
};

fn default_str(field: &str) -> String {
    DEFAULT_PATHS_CONFIG[field]
        .as_str()
        .expect("default field not a string")
        .to_string()
}

fn cfg_file(home: &TempDir) -> PathBuf {
    home.path().join(".config").join("rml").join("paths.json")
}

#[test]
fn default_paths_config_shape() {
    assert_eq!(DEFAULT_PATHS_CONFIG["versions_path"], "~/rml/versions");
    assert_eq!(DEFAULT_PATHS_CONFIG["libs_path"], "~/rml/libs");
    assert_eq!(DEFAULT_PATHS_CONFIG["assets_path"], "~/rml/assets");
    assert_eq!(DEFAULT_PATHS_CONFIG["auth_path"], "~/rml/auth");
}

#[test]
fn paths_load_creates_default_file() {
    let home = TempDir::new("cfg_load");
    let _guard = HomeGuard::new(home.path());

    let paths = Paths::load().unwrap();
    assert_eq!(paths.versions, default_str("versions_path"));
    assert_eq!(paths.libs, default_str("libs_path"));
    assert_eq!(paths.assets, default_str("assets_path"));
    assert_eq!(paths.auth, default_str("auth_path"));

    let f = cfg_file(&home);
    assert!(f.exists(), "config file should be created");
}

#[test]
fn paths_load_is_idempotent() {
    let home = TempDir::new("cfg_load_idem");
    let _guard = HomeGuard::new(home.path());

    let a = Paths::load().unwrap();
    let b = Paths::load().unwrap();
    assert_eq!(a.versions, b.versions);
    assert_eq!(a.libs, b.libs);
    assert_eq!(a.assets, b.assets);
    assert_eq!(a.auth, b.auth);
}

#[test]
fn paths_init_without_force_keeps_existing() {
    let home = TempDir::new("cfg_init_keep");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/custom/versions","libs_path":"/custom/libs","assets_path":"/custom/assets","auth_path":"/custom/auth"}"#,
    )
    .unwrap();

    Paths::init(false).unwrap();

    let paths = Paths::load().unwrap();
    assert_eq!(paths.versions, "/custom/versions");
    assert_eq!(paths.libs, "/custom/libs");
    assert_eq!(paths.assets, "/custom/assets");
    assert_eq!(paths.auth, "/custom/auth");
}

#[test]
fn paths_init_force_overwrites_existing() {
    let home = TempDir::new("cfg_init_force");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/custom/versions","libs_path":"/custom/libs","assets_path":"/custom/assets","auth_path":"/custom/auth"}"#,
    )
    .unwrap();

    Paths::init(true).unwrap();

    let paths = Paths::load().unwrap();
    assert_eq!(paths.versions, default_str("versions_path"));
    assert_eq!(paths.libs, default_str("libs_path"));
    assert_eq!(paths.assets, default_str("assets_path"));
    assert_eq!(paths.auth, default_str("auth_path"));
}

#[test]
fn paths_load_soft_migrates_missing_auth() {
    let home = TempDir::new("cfg_soft_migrate");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a"}"#,
    )
    .unwrap();

    let p = Paths::load().unwrap();
    assert_eq!(p.versions, "/v");
    assert_eq!(p.libs, "/l");
    assert_eq!(p.assets, "/a");
    assert_eq!(p.auth, "~/rml/auth");

    let contents = fs::read_to_string(&f).unwrap();
    assert!(
        !contents.contains("auth_path"),
        "load must not rewrite the file"
    );
}

#[test]
fn paths_load_rejects_wrong_type_auth() {
    let home = TempDir::new("cfg_auth_wrong_type");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a","auth_path":42}"#,
    )
    .unwrap();

    let err = Paths::load().unwrap_err();
    assert!(
        matches!(err, ConfigError::TypeMismatch { .. }),
        "got: {err:?}"
    );
}

#[test]
fn paths_init_soft_migrates_missing_auth() {
    let home = TempDir::new("cfg_init_migrate_auth");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a"}"#,
    )
    .unwrap();

    Paths::init(false).unwrap();

    let p = Paths::load().unwrap();
    assert_eq!(p.versions, "/v");
    assert_eq!(p.libs, "/l");
    assert_eq!(p.assets, "/a");
    assert_eq!(p.auth, "~/rml/auth");

    let contents = fs::read_to_string(&f).unwrap();
    assert!(
        contents.contains("auth_path"),
        "init must persist auth_path"
    );
    assert!(contents.contains("/v"), "existing values must be preserved");
}

#[test]
fn paths_init_soft_migrates_all_missing() {
    let home = TempDir::new("cfg_init_migrate_all");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(&f, br#"{"versions_path":"/v"}"#).unwrap();

    Paths::init(false).unwrap();

    let p = Paths::load().unwrap();
    assert_eq!(p.versions, "/v");
    assert_eq!(p.libs, default_str("libs_path"));
    assert_eq!(p.assets, default_str("assets_path"));
    assert_eq!(p.auth, default_str("auth_path"));
}

#[test]
fn paths_init_valid_config_is_unchanged() {
    let home = TempDir::new("cfg_init_noop");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    let original =
        br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a","auth_path":"/auth"}"#;
    fs::write(&f, original).unwrap();

    Paths::init(false).unwrap();

    let after = fs::read(&f).unwrap();
    assert_eq!(after, original, "valid config must not be rewritten");
}

#[test]
fn paths_init_rejects_wrong_typed_auth() {
    let home = TempDir::new("cfg_init_wrong_type");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    let original = br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a","auth_path":42}"#;
    fs::write(&f, original).unwrap();

    let err = Paths::init(false).unwrap_err();
    assert!(
        matches!(err, ConfigError::TypeMismatch { .. }),
        "got: {err:?}"
    );

    let after = fs::read(&f).unwrap();
    assert_eq!(after, original, "file must not be modified on error");
}

#[test]
fn paths_pathbuf_expands_tilde() {
    let home = TempDir::new("cfg_pathbuf");
    let _guard = HomeGuard::new(home.path());

    let paths = Paths::load().unwrap();
    let versions = paths.versions_pathbuf();
    let libs = paths.libs_pathbuf();
    let assets = paths.assets_pathbuf();
    let auth = paths.auth_pathbuf();

    let home_path = home.path();
    assert_eq!(versions, home_path.join("rml").join("versions"));
    assert_eq!(libs, home_path.join("rml").join("libs"));
    assert_eq!(assets, home_path.join("rml").join("assets"));
    assert_eq!(auth, home_path.join("rml").join("auth"));
}

#[test]
fn get_versions_path_returns_default() {
    let home = TempDir::new("cfg_get_versions");
    let _guard = HomeGuard::new(home.path());

    assert_eq!(get_versions_path(), default_str("versions_path"));
}

#[test]
fn get_libs_path_returns_default() {
    let home = TempDir::new("cfg_get_libs");
    let _guard = HomeGuard::new(home.path());

    assert_eq!(get_libs_path(), default_str("libs_path"));
}

#[test]
fn get_assets_path_returns_default() {
    let home = TempDir::new("cfg_get_assets");
    let _guard = HomeGuard::new(home.path());

    assert_eq!(get_assets_path(), default_str("assets_path"));
}

#[test]
fn get_auth_path_returns_default() {
    let home = TempDir::new("cfg_get_auth");
    let _guard = HomeGuard::new(home.path());

    assert_eq!(get_auth_path(), default_str("auth_path"));
}

#[test]
fn config_init_does_not_panic() {
    let home = TempDir::new("cfg_init_smoke");
    let _guard = HomeGuard::new(home.path());

    init(false);
    init(true);

    let f = cfg_file(&home);
    assert!(f.exists());
}

#[test]
fn paths_struct_is_clone_debug() {
    let p = Paths {
        versions: "a".to_string(),
        libs: "b".to_string(),
        assets: "c".to_string(),
        auth: "d".to_string(),
    };
    let q = p.clone();
    assert_eq!(p.versions, q.versions);
    assert_eq!(p.auth, q.auth);
    let _ = format!("{:?}", p);
}

#[test]
fn paths_pathbuf_relative_path_kept_relative() {
    let home = TempDir::new("cfg_pathbuf_rel");
    let _guard = HomeGuard::new(home.path());

    let p = Paths {
        versions: "relative/versions".to_string(),
        libs: "relative/libs".to_string(),
        assets: "relative/assets".to_string(),
        auth: "relative/auth".to_string(),
    };
    let v = p.versions_pathbuf();
    assert!(v.is_absolute());
    assert!(v.ends_with("relative/versions"));
    let a = p.auth_pathbuf();
    assert!(a.is_absolute());
    assert!(a.ends_with("relative/auth"));
}

#[test]
fn check_config_ok_after_init() {
    let home = TempDir::new("check_cfg_ok");
    let _guard = HomeGuard::new(home.path());

    Paths::init(false).unwrap();
    assert!(check_config().is_ok());
}

#[test]
fn check_config_err_when_file_missing() {
    let home = TempDir::new("check_cfg_missing_file");
    let _guard = HomeGuard::new(home.path());

    let err = check_config().unwrap_err();
    assert!(matches!(err, ConfigError::Io(_)), "got: {err:?}");
}

#[test]
fn check_config_err_on_invalid_json() {
    let home = TempDir::new("check_cfg_bad_json");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(&f, b"not json").unwrap();

    let err = check_config().unwrap_err();
    assert!(matches!(err, ConfigError::Parse(_)), "got: {err:?}");
}

#[test]
fn check_config_err_when_auth_missing() {
    let home = TempDir::new("check_cfg_missing_auth");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a"}"#,
    )
    .unwrap();

    let err = check_config().unwrap_err();
    match err {
        ConfigError::KeyNotFound(k) => assert_eq!(k, "auth_path"),
        other => panic!("expected KeyNotFound, got: {other:?}"),
    }
}

#[test]
fn check_config_err_when_versions_missing() {
    let home = TempDir::new("check_cfg_missing_versions");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"libs_path":"/l","assets_path":"/a","auth_path":"/auth"}"#,
    )
    .unwrap();

    let err = check_config().unwrap_err();
    match err {
        ConfigError::KeyNotFound(k) => assert_eq!(k, "versions_path"),
        other => panic!("expected KeyNotFound, got: {other:?}"),
    }
}

#[test]
fn check_config_err_on_wrong_type() {
    let home = TempDir::new("check_cfg_wrong_type");
    let _guard = HomeGuard::new(home.path());

    let f = cfg_file(&home);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(
        &f,
        br#"{"versions_path":"/v","libs_path":"/l","assets_path":"/a","auth_path":42}"#,
    )
    .unwrap();

    let err = check_config().unwrap_err();
    assert!(
        matches!(err, ConfigError::TypeMismatch { .. }),
        "got: {err:?}"
    );
}
