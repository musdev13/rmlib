#![cfg(feature = "modrinth")]

mod common;

use common::{HomeGuard, TempDir};

use rmlib::extra::modrinth::dto::*;
use rmlib::extra::modrinth::{
    FacetBuilder, ModrinthError, build_query, normalize_limit, page_to_offset,
};
use rmlib::extra::modrinth::{hash, install, version};

#[test]
fn facet_single_type() {
    let f = FacetBuilder::new().project_type(ProjectType::Mod);
    assert_eq!(f.build(), r#"[["project_type:mod"]]"#);
}

#[test]
fn facet_multiple_versions_grouped() {
    let f = FacetBuilder::new()
        .project_type(ProjectType::Mod)
        .game_version("1.20.1")
        .game_version("1.21");
    let out = f.build();
    assert!(out.contains(r#""project_type:mod""#));
    assert!(out.contains(r#""versions:1.20.1""#));
    assert!(out.contains(r#""versions:1.21""#));
}

#[test]
fn page_to_offset_basic() {
    assert_eq!(page_to_offset(1, 20).unwrap(), 0);
    assert_eq!(page_to_offset(3, 20).unwrap(), 40);
}

#[test]
fn page_to_offset_rejects_zero() {
    assert!(page_to_offset(0, 20).is_err());
    assert!(page_to_offset(1, 0).is_err());
}

#[test]
fn page_to_offset_overflow() {
    assert!(page_to_offset(u32::MAX, u32::MAX).is_err());
}

#[test]
fn normalize_limit_clamps() {
    assert_eq!(normalize_limit(0), 1);
    assert_eq!(normalize_limit(1000), 100);
    assert_eq!(normalize_limit(50), 50);
}

#[test]
fn build_query_skips_empty() {
    let p = SearchParams {
        query: Some(String::new()),
        facets: None,
        index: None,
        offset: None,
        limit: None,
    };
    let q = build_query(&p);
    assert!(q.is_empty());
}

#[test]
fn build_query_full() {
    let p = SearchParams {
        query: Some("sodium".to_string()),
        facets: Some("[[\"project_type:mod\"]]".to_string()),
        index: Some(SearchIndex::Downloads),
        offset: Some(40),
        limit: Some(20),
    };
    let q = build_query(&p);
    let map: std::collections::HashMap<_, _> = q.into_iter().collect();
    assert_eq!(map["query"], "sodium");
    assert_eq!(map["index"], "downloads");
    assert_eq!(map["offset"], "40");
    assert_eq!(map["limit"], "20");
}

#[test]
fn search_index_from_str_known() {
    assert!(matches!(
        rmlib::extra::modrinth::search_index_from_str("relevance"),
        Some(SearchIndex::Relevance)
    ));
    assert!(rmlib::extra::modrinth::search_index_from_str("bogus").is_none());
}

#[test]
fn project_type_dirs() {
    assert_eq!(ProjectType::Mod.as_dir(), Some("mods"));
    assert_eq!(ProjectType::Resourcepack.as_dir(), Some("resourcepacks"));
    assert_eq!(ProjectType::Shader.as_dir(), Some("shaderpacks"));
    assert_eq!(ProjectType::Datapack.as_dir(), Some("datapacks"));
    assert_eq!(ProjectType::Modpack.as_dir(), None);
}

#[test]
fn hash_sha1_stable() {
    let h = hash::sha1_bytes(b"hello");
    assert_eq!(h, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
}

#[test]
fn hash_sha512_stable() {
    let h = hash::sha512_bytes(b"hello");
    assert_eq!(h.len(), 128);
    assert!(h.starts_with("9b71d224bd62f3785d96d46ad3ea3d73"));
}

#[test]
fn verify_sha1_mismatch() {
    let dir = TempDir::new("mrm_verify_sha1");
    let _guard = HomeGuard::new(dir.path());
    let file = dir.write("f.bin", b"hello");
    assert!(hash::verify_sha1(&file, &hash::sha1_bytes(b"hello")).is_ok());
    match hash::verify_sha1(&file, "wrong") {
        Err(ModrinthError::HashMismatch { expected, .. }) => assert_eq!(expected, "wrong"),
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

#[test]
fn version_selection_by_filters() {
    let mk = |vtype: &str, mc: &str, loader: &str| Version {
        id: format!("{vtype}-{mc}-{loader}"),
        project_id: "p".to_string(),
        version_type: vtype.to_string(),
        game_versions: vec![mc.to_string()],
        loaders: vec![loader.to_string()],
        ..Default::default()
    };
    let versions = vec![
        mk("release", "1.20.1", "fabric"),
        mk("beta", "1.20.1", "fabric"),
        mk("release", "1.19.4", "fabric"),
    ];
    let sel = version::select_compatible_version(
        &versions,
        Some("1.20.1"),
        Some("fabric"),
        Some(VersionType::Release),
    )
    .unwrap();
    assert_eq!(sel.id, "release-1.20.1-fabric");

    let sel = version::select_compatible_version(
        &versions,
        Some("1.20.1"),
        Some("fabric"),
        Some(VersionType::Beta),
    )
    .unwrap();
    assert_eq!(sel.id, "beta-1.20.1-fabric");

    let none = version::select_compatible_version(&versions, Some("1.18"), Some("fabric"), None);
    assert!(none.is_none());
}

#[test]
fn filter_versions_by_loader() {
    let v1 = Version {
        id: "1".to_string(),
        version_type: "release".to_string(),
        game_versions: vec!["1.20.1".to_string()],
        loaders: vec!["fabric".to_string()],
        ..Default::default()
    };
    let v2 = Version {
        id: "2".to_string(),
        version_type: "release".to_string(),
        game_versions: vec!["1.20.1".to_string()],
        loaders: vec!["forge".to_string()],
        ..Default::default()
    };
    let out = version::filter_versions(
        &[v1, v2],
        &VersionFilters {
            game_versions: None,
            loaders: Some(vec!["fabric".to_string()]),
            version_type: None,
        },
    );
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "1");
}

#[test]
fn resolve_install_dir_auto() {
    let params = InstallParams {
        game_path: Some(std::path::PathBuf::from("/game")),
        ..Default::default()
    };
    let dir = install::resolve_install_dir(ProjectType::Mod, &params).unwrap();
    assert_eq!(dir, std::path::PathBuf::from("/game/mods"));
    let dir = install::resolve_install_dir(ProjectType::Shader, &params).unwrap();
    assert_eq!(dir, std::path::PathBuf::from("/game/shaderpacks"));
}

#[test]
fn resolve_install_dir_explicit_wins() {
    let params = InstallParams {
        game_path: Some(std::path::PathBuf::from("/game")),
        install_dir: Some(std::path::PathBuf::from("/custom")),
        ..Default::default()
    };
    let dir = install::resolve_install_dir(ProjectType::Mod, &params).unwrap();
    assert_eq!(dir, std::path::PathBuf::from("/custom"));
}

#[test]
fn resolve_install_dir_missing_game_path() {
    let params = InstallParams::default();
    assert!(install::resolve_install_dir(ProjectType::Mod, &params).is_err());
}

#[test]
fn resolve_install_dir_modpack_requires_explicit() {
    let params = InstallParams {
        game_path: Some(std::path::PathBuf::from("/game")),
        ..Default::default()
    };
    assert!(install::resolve_install_dir(ProjectType::Modpack, &params).is_err());
}

#[test]
fn parse_project_type_known() {
    assert_eq!(install::parse_project_type("mod"), ProjectType::Mod);
    assert_eq!(install::parse_project_type("modpack"), ProjectType::Modpack);
    assert_eq!(install::parse_project_type("shader"), ProjectType::Shader);
}

#[test]
fn world_name_validation() {
    assert!(install::validate_world_name("MyWorld").is_ok());
    assert!(install::validate_world_name("").is_err());
    assert!(install::validate_world_name("..").is_err());
    assert!(install::validate_world_name("a/b").is_err());
}

#[test]
fn project_deserialization_optional_fields() {
    let raw = r#"{
        "id": "p1", "slug": "sodium", "title": "Sodium",
        "description": "d", "project_type": "mod", "downloads": 10,
        "followers": 5, "versions": ["v1"], "game_versions": ["1.20.1"],
        "loaders": ["fabric"], "team": "t1"
    }"#;
    let p: Project = serde_json::from_str(raw).unwrap();
    assert_eq!(p.slug, "sodium");
    assert_eq!(p.loaders, vec!["fabric".to_string()]);
    assert!(p.icon_url.is_none());
    assert!(p.license.is_none());
}

#[test]
fn search_response_deserialization() {
    let raw = r#"{
        "hits": [{
            "project_id": "p1", "project_type": "mod", "slug": "s",
            "author": "a", "title": "T", "description": "d",
            "categories": [], "display_categories": [], "versions": ["v1"],
            "downloads": 1, "follows": 2,
            "date_created": "d", "date_modified": "m"
        }],
        "offset": 0, "limit": 10, "total_hits": 1
    }"#;
    let r: SearchResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(r.hits.len(), 1);
    assert_eq!(r.total_hits, 1);
    assert!(r.hits[0].icon_url.is_none());
}

#[test]
fn version_primary_file_fallback() {
    let v = Version {
        files: vec![
            VersionFile {
                filename: "a.jar".to_string(),
                primary: false,
                ..Default::default()
            },
            VersionFile {
                filename: "b.jar".to_string(),
                primary: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert_eq!(v.primary_file().unwrap().filename, "b.jar");

    let v2 = Version {
        files: vec![VersionFile {
            filename: "a.jar".to_string(),
            primary: false,
            ..Default::default()
        }],
        ..Default::default()
    };
    assert_eq!(v2.primary_file().unwrap().filename, "a.jar");
    let empty = Version::default();
    assert!(empty.primary_file().is_none());
}

#[test]
fn dependency_helpers() {
    let mk = |t: &str| Dependency {
        dependency_type: t.to_string(),
        ..Default::default()
    };
    assert!(mk("required").is_required());
    assert!(mk("optional").is_optional());
    assert!(mk("incompatible").is_incompatible());
    assert!(mk("embedded").is_embedded());
}

#[test]
fn version_type_helpers() {
    let v = Version {
        version_type: "release".to_string(),
        ..Default::default()
    };
    assert!(v.is_release());
    let v = Version {
        version_type: "beta".to_string(),
        ..Default::default()
    };
    assert!(v.is_beta());
    let v = Version {
        version_type: "alpha".to_string(),
        ..Default::default()
    };
    assert!(v.is_alpha());
}

#[test]
fn error_display() {
    let e = ModrinthError::HashMismatch {
        expected: "a".to_string(),
        actual: "b".to_string(),
    };
    assert!(e.to_string().contains("hash mismatch"));
    let e = ModrinthError::NoCompatibleVersion("x".to_string());
    assert!(e.to_string().contains("no compatible version"));
    let e = ModrinthError::RateLimited {
        retry_after: Some(5),
    };
    assert!(e.to_string().contains("5s"));
}
