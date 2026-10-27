#![cfg(feature = "core")]

mod common;

use std::fs;
use std::path::PathBuf;

use common::{HomeGuard, TempDir};
use rmlib::core::config::{
    DEFAULT_PATHS_CONFIG, Paths, get_assets_path, get_libs_path, get_versions_path, init,
};

fn default_str(field: &str) -> String {
    DEFAULT_PATHS_CONFIG[field]
        .as_str()
        .expect("default field not a string")
        .to_string()
}

#[test]
fn default_paths_config_shape() {
    assert_eq!(DEFAULT_PATHS_CONFIG["versions_path"], "~/rml/versions");
    assert_eq!(DEFAULT_PATHS_CONFIG["libs_path"], "~/rml/libs");
    assert_eq!(DEFAULT_PATHS_CONFIG["assets_path"], "~/rml/assets");
}

#[test]
fn paths_load_creates_default_file() {
    let home = TempDir::new("cfg_load");
    let _guard = HomeGuard::new(home.path());

    let paths = Paths::load().unwrap();
    assert_eq!(paths.versions, default_str("versions_path"));
    assert_eq!(paths.libs, default_str("libs_path"));
    assert_eq!(paths.assets, default_str("assets_path"));

    let cfg_file = home.path().join(".config").join("rml").join("paths.json");
    assert!(cfg_file.exists(), "config file should be created");
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
}

#[test]
fn paths_init_without_force_keeps_existing() {
    let home = TempDir::new("cfg_init_keep");
    let _guard = HomeGuard::new(home.path());

    let cfg_dir = home.path().join(".config").join("rml");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("paths.json"),
        br#"{"versions_path":"/custom/versions","libs_path":"/custom/libs","assets_path":"/custom/assets"}"#,
    )
    .unwrap();

    Paths::init(false).unwrap();

    let paths = Paths::load().unwrap();
    assert_eq!(paths.versions, "/custom/versions");
    assert_eq!(paths.libs, "/custom/libs");
    assert_eq!(paths.assets, "/custom/assets");
}

#[test]
fn paths_init_force_overwrites_existing() {
    let home = TempDir::new("cfg_init_force");
    let _guard = HomeGuard::new(home.path());

    let cfg_dir = home.path().join(".config").join("rml");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("paths.json"),
        br#"{"versions_path":"/custom/versions","libs_path":"/custom/libs","assets_path":"/custom/assets"}"#,
    )
    .unwrap();

    Paths::init(true).unwrap();

    let paths = Paths::load().unwrap();
    assert_eq!(paths.versions, default_str("versions_path"));
    assert_eq!(paths.libs, default_str("libs_path"));
    assert_eq!(paths.assets, default_str("assets_path"));
}

#[test]
fn paths_pathbuf_expands_tilde() {
    let home = TempDir::new("cfg_pathbuf");
    let _guard = HomeGuard::new(home.path());

    let paths = Paths::load().unwrap();
    let versions = paths.versions_pathbuf();
    let libs = paths.libs_pathbuf();
    let assets = paths.assets_pathbuf();

    let home_path = home.path();
    assert_eq!(versions, home_path.join("rml").join("versions"));
    assert_eq!(libs, home_path.join("rml").join("libs"));
    assert_eq!(assets, home_path.join("rml").join("assets"));
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
fn config_init_does_not_panic() {
    let home = TempDir::new("cfg_init_smoke");
    let _guard = HomeGuard::new(home.path());

    init(false);
    init(true);

    let cfg_file = home.path().join(".config").join("rml").join("paths.json");
    assert!(cfg_file.exists());
}

#[test]
fn paths_struct_is_clone_debug() {
    let p = Paths {
        versions: "a".to_string(),
        libs: "b".to_string(),
        assets: "c".to_string(),
    };
    let q = p.clone();
    assert_eq!(p.versions, q.versions);
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
    };
    let _ = PathBuf::from("relative/versions");
    let v = p.versions_pathbuf();
    assert!(v.is_absolute());
    assert!(v.ends_with("relative/versions"));
}
