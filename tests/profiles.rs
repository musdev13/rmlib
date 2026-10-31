#![cfg(feature = "profiles")]

mod common;

use std::fs;

use common::{HomeGuard, TempDir};
use rmlib::extra::profiles::{
    InstanceConfig, Profile, ProfileError, ProfileKind, delete_profile_at, instance_default_path,
    list_profiles_at, load_instance, load_profile_at, profile_path_at, profiles_dir_at,
    save_instance, save_profile_at,
};

fn sample(kind: ProfileKind) -> Profile {
    match kind {
        ProfileKind::Ely => Profile {
            username: "Musek".to_string(),
            uuid: Some("ely-uuid".to_string()),
            access_token: "ely-access".to_string(),
            client_token: Some("ely-client".to_string()),
            xuid: None,
            has_license: None,
        },
        ProfileKind::Ms => Profile {
            username: "Musek".to_string(),
            uuid: Some("ms-uuid".to_string()),
            access_token: "ms-access".to_string(),
            client_token: None,
            xuid: Some("ms-xuid".to_string()),
            has_license: Some(true),
        },
    }
}

#[test]
fn profile_path_layout() {
    let home = TempDir::new("prof_path");
    let _guard = HomeGuard::new(home.path());

    let p = profile_path_at(None, ProfileKind::Ely, "Musek").unwrap();
    assert_eq!(
        p,
        home.path()
            .join("rml")
            .join("auth")
            .join("profiles")
            .join("ely")
            .join("Musek.json")
    );
}

#[test]
fn profiles_dir_respects_kind() {
    let home = TempDir::new("prof_dir_kind");
    let _guard = HomeGuard::new(home.path());

    let ely = profiles_dir_at(None, ProfileKind::Ely).unwrap();
    let ms = profiles_dir_at(None, ProfileKind::Ms).unwrap();
    assert!(ely.ends_with("profiles/ely"));
    assert!(ms.ends_with("profiles/ms"));
}

#[test]
fn save_load_roundtrip_ely() {
    let home = TempDir::new("prof_roundtrip_ely");
    let _guard = HomeGuard::new(home.path());

    let profile = sample(ProfileKind::Ely);
    let path = save_profile_at(None, ProfileKind::Ely, &profile).unwrap();
    assert!(path.exists());

    let loaded = load_profile_at(None, ProfileKind::Ely, "Musek")
        .unwrap()
        .expect("profile");
    assert_eq!(loaded, profile);
}

#[test]
fn save_load_roundtrip_ms() {
    let home = TempDir::new("prof_roundtrip_ms");
    let _guard = HomeGuard::new(home.path());

    let profile = sample(ProfileKind::Ms);
    save_profile_at(None, ProfileKind::Ms, &profile).unwrap();
    let loaded = load_profile_at(None, ProfileKind::Ms, "Musek")
        .unwrap()
        .expect("profile");
    assert_eq!(loaded, profile);
}

#[test]
fn ely_and_ms_are_separate() {
    let home = TempDir::new("prof_separate");
    let _guard = HomeGuard::new(home.path());

    save_profile_at(None, ProfileKind::Ely, &sample(ProfileKind::Ely)).unwrap();
    assert!(
        load_profile_at(None, ProfileKind::Ms, "Musek")
            .unwrap()
            .is_none()
    );
}

#[test]
fn load_missing_returns_none() {
    let home = TempDir::new("prof_missing");
    let _guard = HomeGuard::new(home.path());

    assert!(
        load_profile_at(None, ProfileKind::Ely, "Nobody")
            .unwrap()
            .is_none()
    );
}

#[test]
fn delete_removes_only_target() {
    let home = TempDir::new("prof_delete");
    let _guard = HomeGuard::new(home.path());

    save_profile_at(None, ProfileKind::Ely, &sample(ProfileKind::Ely)).unwrap();
    let other = Profile {
        username: "Other".to_string(),
        ..sample(ProfileKind::Ely)
    };
    save_profile_at(None, ProfileKind::Ely, &other).unwrap();

    delete_profile_at(None, ProfileKind::Ely, "Musek").unwrap();
    assert!(
        load_profile_at(None, ProfileKind::Ely, "Musek")
            .unwrap()
            .is_none()
    );
    assert!(
        load_profile_at(None, ProfileKind::Ely, "Other")
            .unwrap()
            .is_some()
    );
}

#[test]
fn delete_missing_is_ok() {
    let home = TempDir::new("prof_delete_missing");
    let _guard = HomeGuard::new(home.path());

    delete_profile_at(None, ProfileKind::Ely, "Ghost").unwrap();
}

#[test]
fn list_sorted_and_filtered() {
    let home = TempDir::new("prof_list");
    let _guard = HomeGuard::new(home.path());

    for name in ["bob", "alice", "carol"] {
        let profile = Profile {
            username: name.to_string(),
            ..sample(ProfileKind::Ms)
        };
        save_profile_at(None, ProfileKind::Ms, &profile).unwrap();
    }
    let dir = profiles_dir_at(None, ProfileKind::Ms).unwrap();
    fs::write(dir.join("notjson.txt"), b"x").unwrap();

    let list = list_profiles_at(None, ProfileKind::Ms).unwrap();
    assert_eq!(list, vec!["alice", "bob", "carol"]);
}

#[test]
fn list_empty_when_dir_absent() {
    let home = TempDir::new("prof_list_absent");
    let _guard = HomeGuard::new(home.path());

    assert!(list_profiles_at(None, ProfileKind::Ely).unwrap().is_empty());
}

#[test]
fn custom_auth_dir_is_used() {
    let home = TempDir::new("prof_custom_dir");
    let _guard = HomeGuard::new(home.path());

    let custom = home.join("custom_auth");
    save_profile_at(Some(&custom), ProfileKind::Ms, &sample(ProfileKind::Ms)).unwrap();

    let expected = custom.join("profiles").join("ms").join("Musek.json");
    assert!(expected.exists());
    assert!(
        !home
            .path()
            .join("rml")
            .join("auth")
            .join("profiles")
            .exists()
    );

    let loaded = load_profile_at(Some(&custom), ProfileKind::Ms, "Musek")
        .unwrap()
        .expect("profile");
    assert_eq!(loaded.access_token, "ms-access");
}

#[test]
fn rejects_path_traversal() {
    let home = TempDir::new("prof_traversal");
    let _guard = HomeGuard::new(home.path());

    for bad in ["../evil", "a/b", "a\\b", "..", "."] {
        let err = profile_path_at(None, ProfileKind::Ely, bad).unwrap_err();
        assert!(matches!(err, ProfileError::Io(_)), "username {bad:?}");
    }
}

#[test]
fn corrupt_profile_is_json_error() {
    let home = TempDir::new("prof_corrupt");
    let _guard = HomeGuard::new(home.path());

    let path = profile_path_at(None, ProfileKind::Ely, "Musek").unwrap();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not json").unwrap();

    let err = load_profile_at(None, ProfileKind::Ely, "Musek").unwrap_err();
    assert!(matches!(err, ProfileError::Json(_)));
}

#[test]
fn profile_kind_dir_names() {
    assert_eq!(ProfileKind::Ely.as_dir(), "ely");
    assert_eq!(ProfileKind::Ms.as_dir(), "ms");
}

#[test]
fn profile_serde_omits_optional_none() {
    let value = serde_json::to_value(Profile {
        username: "u".to_string(),
        uuid: None,
        access_token: "t".to_string(),
        client_token: None,
        xuid: None,
        has_license: None,
    })
    .unwrap();
    assert_eq!(value["username"], "u");
    assert_eq!(value["access_token"], "t");
    assert!(value.get("uuid").is_none());
    assert!(value.get("client_token").is_none());
}

#[test]
fn instance_default_path_joins_rml_json() {
    let p = instance_default_path(std::path::Path::new("/game"));
    assert_eq!(p, std::path::PathBuf::from("/game/rml.json"));
}

#[test]
fn instance_save_load_roundtrip() {
    let home = TempDir::new("inst_roundtrip");
    let _guard = HomeGuard::new(home.path());

    let cfg = InstanceConfig {
        version_id: Some("26.3".to_string()),
        ram: Some("6G".to_string()),
        username: Some("Musek".to_string()),
        ely: Some(true),
        betafix: None,
        profile: Some(true),
        jvm_args: Some("-Xmx6G".to_string()),
        game_args: None,
        versions: Some(std::path::PathBuf::from("/v")),
        assets: None,
        libs: None,
        game_path: Some(std::path::PathBuf::from("./game")),
        spawn_path: None,
        java: None,
        fabric: Some("0.20.1".to_string()),
        neoforge: None,
        auth_path: None,
    };
    let path = home.join("rml.json");
    save_instance(&path, &cfg).unwrap();
    let loaded = load_instance(&path).unwrap();
    assert_eq!(loaded, cfg);
}

#[test]
fn instance_json_skips_none() {
    let cfg = InstanceConfig {
        version_id: Some("26.3".to_string()),
        ram: None,
        username: None,
        ely: Some(false),
        betafix: None,
        profile: None,
        jvm_args: None,
        game_args: None,
        versions: None,
        assets: None,
        libs: None,
        game_path: None,
        spawn_path: None,
        java: None,
        fabric: None,
        neoforge: None,
        auth_path: None,
    };
    let value = serde_json::to_value(&cfg).unwrap();
    assert_eq!(value["version_id"], "26.3");
    assert_eq!(value["ely"], false);
    assert!(value.get("ram").is_none());
    assert!(value.get("username").is_none());
}

#[test]
fn instance_sparse_json_loads() {
    let raw = r#"{"version_id":"1.21.1","fabric":"0.16.0"}"#;
    let cfg: InstanceConfig = serde_json::from_str(raw).unwrap();
    assert_eq!(cfg.version_id.as_deref(), Some("1.21.1"));
    assert_eq!(cfg.fabric.as_deref(), Some("0.16.0"));
    assert!(cfg.ram.is_none());
}

#[test]
fn instance_load_missing_errors() {
    let home = TempDir::new("inst_missing");
    let _guard = HomeGuard::new(home.path());

    let path = home.join("nope.json");
    assert!(matches!(load_instance(&path), Err(ProfileError::Io(_))));
}

#[test]
fn instance_corrupt_json_errors() {
    let home = TempDir::new("inst_corrupt");
    let _guard = HomeGuard::new(home.path());

    let path = home.write("bad.json", b"not json");
    assert!(matches!(load_instance(&path), Err(ProfileError::Json(_))));
}

#[test]
fn instance_merge_fills_only_missing() {
    let mut base = InstanceConfig {
        version_id: Some("cli".to_string()),
        ram: None,
        ..Default::default()
    };
    let other = InstanceConfig {
        version_id: Some("file".to_string()),
        ram: Some("4G".to_string()),
        username: Some("FromFile".to_string()),
        ..Default::default()
    };
    base.merge_missing_from(&other);
    assert_eq!(base.version_id.as_deref(), Some("cli"));
    assert_eq!(base.ram.as_deref(), Some("4G"));
    assert_eq!(base.username.as_deref(), Some("FromFile"));
}
