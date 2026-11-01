#![cfg(feature = "resman-mods")]

mod common;

use std::fs;
use std::io::Write;
use std::path::Path;

use common::{HomeGuard, TempDir};
use rmlib::extra::resman::mods::{
    FallbackPolicy, Field, Format, ModError, ModLoader, Priority, Source, disable, enable,
    info_by_id, info_file_as, info_from_file, is_enabled, parse_fields, scan, scan_dir, toggle,
};

fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let f = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let opts: zip::write::SimpleFileOptions = Default::default();
    for (name, data) in entries {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap();
}

fn fabric_jar(path: &Path, id: &str, name: &str) {
    let fmj = format!(
        r#"{{"schemaVersion":1,"id":"{id}","version":"1.0.0+mc1.21.1","name":"{name}","description":"desc {name}","authors":["Author {name}"],"license":"MIT","depends":{{"minecraft":"1.21.1","fabric-loader":">=0.15","fabric-api":"*","sodium":"0.8.x"}},"recommends":{{"modmenu":"*"}}}}"#
    );
    write_zip(path, &[("fabric.mod.json", fmj.as_bytes())]);
}

fn neoforge_jar(path: &Path, id: &str, name: &str) {
    let toml = format!(
        r#"modLoader = "javafml"
loaderVersion = "[1,)"
license = "MIT"
[[mods]]
modId = "{id}"
version = "1.21.1-1.0.0"
displayName = "{name}"
authors = "Author A, Author B"
description = '''desc {name}'''
[[dependencies.{id}]]
modId = "neoforge"
type = "required"
versionRange = "[1.0.0,)"
[[dependencies.{id}]]
modId = "minecraft"
type = "required"
versionRange = "[1.21.1,1.21.2)"
[[dependencies.{id}]]
modId = "commonnetworking"
type = "required"
versionRange = "[1.0.16,)"
[[dependencies.{id}]]
modId = "optional_lib"
type = "optional"
versionRange = "*"
"#
    );
    write_zip(path, &[("META-INF/neoforge.mods.toml", toml.as_bytes())]);
}

fn decoy_neoforge(path: &Path) {
    let decoy = r#"{"schemaVersion":1,"id":"x-wrongloader","version":"not a fabric mod","name":"X","description":"d","authors":[],"entrypoints":{"preLaunch":["x.FabricError"]}}"#;
    let toml = r#"modLoader = "javafml"
loaderVersion = "[1,)"
license = "All rights reserved"
[[mods]]
modId = "x"
version = "1.0.0"
displayName = "X"
description = "d"
"#;
    write_zip(
        path,
        &[
            ("fabric.mod.json", decoy.as_bytes()),
            ("META-INF/neoforge.mods.toml", toml.as_bytes()),
        ],
    );
}

#[test]
fn scan_finds_fabric_and_neoforge() {
    let home = TempDir::new("rm_scan");
    let _guard = HomeGuard::new(home.path());
    let game = home.join("game");
    fabric_jar(&game.join("mods").join("sodium.jar"), "sodium", "Sodium");
    neoforge_jar(&game.join("mods").join("jei.jar"), "jei", "JEI");

    let list = scan(&game).unwrap();
    assert_eq!(list.len(), 2);
    let ids: Vec<_> = list.iter().filter_map(|m| m.id.clone()).collect();
    assert!(ids.contains(&"sodium".to_string()));
    assert!(ids.contains(&"jei".to_string()));
}

#[test]
fn fabric_info_parsed() {
    let home = TempDir::new("rm_fabric_info");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("sodium.jar");
    fabric_jar(&p, "sodium", "Sodium");

    let info = info_from_file(&p).unwrap();
    assert_eq!(info.id.as_deref(), Some("sodium"));
    assert_eq!(info.loader, Some(ModLoader::Fabric));
    assert_eq!(info.minecraft, Some(vec!["1.21.1".to_string()]));
    assert!(
        info.dependencies
            .iter()
            .any(|d| d.id == "sodium" && d.kind == "required")
    );
    assert!(
        info.dependencies
            .iter()
            .any(|d| d.fabric_api && d.id == "fabric-api")
    );
    assert!(
        info.dependencies
            .iter()
            .any(|d| d.kind == "optional" && d.id == "modmenu")
    );
}

#[test]
fn neoforge_info_parsed() {
    let home = TempDir::new("rm_neoforge_info");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("jei.jar");
    neoforge_jar(&p, "jei", "JEI");

    let info = info_from_file(&p).unwrap();
    assert_eq!(info.id.as_deref(), Some("jei"));
    assert_eq!(info.loader, Some(ModLoader::NeoForge));
    assert_eq!(info.minecraft, Some(vec!["[1.21.1,1.21.2)".to_string()]));
    assert!(info.dependencies.iter().any(|d| d.id == "commonnetworking"));
    assert!(
        info.dependencies
            .iter()
            .any(|d| d.id == "optional_lib" && d.kind == "optional")
    );
}

#[test]
fn decoy_fabric_ignored() {
    let home = TempDir::new("rm_decoy");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("x.jar");
    decoy_neoforge(&p);

    let info = info_from_file(&p).unwrap();
    assert_eq!(info.loader, Some(ModLoader::NeoForge));
    assert_eq!(info.id.as_deref(), Some("x"));
}

#[test]
fn toggle_roundtrip() {
    let home = TempDir::new("rm_toggle");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("m.jar");
    fabric_jar(&p, "m", "M");
    assert!(is_enabled(&p));

    let (p2, enabled) = toggle(&p).unwrap();
    assert!(!enabled);
    assert!(p2.to_string_lossy().ends_with(".jar.disabled"));

    let p3 = enable(&p2).unwrap();
    assert!(is_enabled(&p3));
    let p4 = disable(&p3).unwrap();
    assert!(!is_enabled(&p4));
}

#[test]
fn scan_includes_disabled() {
    let home = TempDir::new("rm_scan_disabled");
    let _guard = HomeGuard::new(home.path());
    let game = home.join("game");
    fabric_jar(&game.join("mods").join("a.jar"), "a", "A");
    fabric_jar(&game.join("mods").join("b.jar.disabled"), "b", "B");

    let list = scan(&game).unwrap();
    assert_eq!(list.len(), 2);
    let a = list.iter().find(|m| m.id.as_deref() == Some("a")).unwrap();
    let b = list.iter().find(|m| m.id.as_deref() == Some("b")).unwrap();
    assert!(a.enabled);
    assert!(!b.enabled);
}

#[test]
fn info_by_id_finds_and_misses() {
    let home = TempDir::new("rm_info_id");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("mods");
    fabric_jar(&dir.join("sodium.jar"), "sodium", "Sodium");

    assert!(info_by_id(&dir, "sodium").unwrap().is_some());
    assert!(info_by_id(&dir, "nope").unwrap().is_none());
}

#[test]
fn priority_parse_ok() {
    let p = Priority::parse("local").unwrap();
    assert!(p.includes(Source::Local));
    assert!(!p.includes(Source::Modrinth));
}

#[test]
fn priority_parse_rejects_unknown() {
    assert!(matches!(
        Priority::parse("bogus"),
        Err(ModError::InvalidArgument(_))
    ));
    assert!(matches!(
        Priority::parse(""),
        Err(ModError::InvalidArgument(_))
    ));
}

#[test]
fn priority_modrinth_gated() {
    let r = Priority::parse("modrinth");
    if cfg!(feature = "modrinth") {
        assert!(r.is_ok());
    } else {
        assert!(matches!(r, Err(ModError::InvalidArgument(_))));
    }
}

#[test]
fn fallback_placeholder() {
    let home = TempDir::new("rm_fallback_placeholder");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("broken.jar");
    fs::write(&p, b"not a zip").unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let priority = Priority::parse("local").unwrap();
    let info = rt
        .block_on(rmlib::extra::resman::mods::info_priority(
            &p,
            &priority,
            &FallbackPolicy::Placeholder,
        ))
        .unwrap()
        .unwrap();
    assert_eq!(info.filename, "broken.jar");
    assert!(info.id.is_none());
}

#[test]
fn fallback_skip() {
    let home = TempDir::new("rm_fallback_skip");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("broken.jar");
    fs::write(&p, b"not a zip").unwrap();

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let priority = Priority::parse("local").unwrap();
    let res = rt
        .block_on(rmlib::extra::resman::mods::info_priority(
            &p,
            &priority,
            &FallbackPolicy::Skip,
        ))
        .unwrap();
    assert!(res.is_none());
}

#[test]
fn fields_parse() {
    let f = parse_fields("id,name,version").unwrap();
    assert_eq!(f, vec![Field::Id, Field::Name, Field::Version]);
    assert!(parse_fields("bogus").is_err());
}

#[test]
fn format_json_and_plain() {
    let home = TempDir::new("rm_format");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("mods");
    fabric_jar(&dir.join("sodium.jar"), "sodium", "Sodium");

    let json = info_file_as(&dir.join("sodium.jar"), None, Format::Json).unwrap();
    assert!(json.contains("\"sodium\""));

    let plain = info_file_as(
        &dir.join("sodium.jar"),
        Some(&[Field::Id, Field::Name]),
        Format::Plain,
    )
    .unwrap();
    assert!(plain.contains("id: sodium"));
    assert!(plain.contains("name: Sodium"));
}

#[test]
fn missing_file_is_not_found() {
    let home = TempDir::new("rm_missing");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("nope.jar");
    assert!(matches!(info_from_file(&p), Err(ModError::NotFound(_))));
}

#[test]
fn empty_dir_scan_ok() {
    let home = TempDir::new("rm_empty");
    let _guard = HomeGuard::new(home.path());
    assert!(scan(&home.join("game")).unwrap().is_empty());
    assert!(scan_dir(&home.join("no_such_dir")).unwrap().is_empty());
}

#[test]
fn universal_mod_detected() {
    let home = TempDir::new("rm_universal");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("lambdynlights.jar");

    let fmj = r#"{"schemaVersion":1,"id":"lambdynlights","version":"4.13.0","name":"LambDynamicLights","description":"d","authors":["LambdAurora"],"license":"Lambda License"}"#;
    let toml = r#"modLoader = "javafml"
loaderVersion = "[1,)"
license = "Lambda License"
[[mods]]
modId = "lambdynlights"
version = "4.13.0"
displayName = "LambDynamicLights"
description = "d"
authors = "LambdAurora"
"#;
    write_zip(
        &p,
        &[
            ("fabric.mod.json", fmj.as_bytes()),
            ("META-INF/neoforge.mods.toml", toml.as_bytes()),
        ],
    );

    let info = info_from_file(&p).unwrap();
    assert_eq!(info.id.as_deref(), Some("lambdynlights"));
    assert_eq!(info.loader, Some(ModLoader::Universal));
}

#[test]
fn decoy_still_reports_neoforge() {
    let home = TempDir::new("rm_decoy_not_universal");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("x.jar");
    decoy_neoforge(&p);
    let info = info_from_file(&p).unwrap();
    assert_eq!(info.loader, Some(ModLoader::NeoForge));
}

#[test]
fn scan_priority_dir_uses_dir_directly() {
    let home = TempDir::new("rm_scan_priority_dir");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("mods");
    fabric_jar(&dir.join("a.jar"), "a", "A");
    fabric_jar(&dir.join("b.jar"), "b", "B");

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let priority = Priority::parse("local").unwrap();
    let list = rt
        .block_on(rmlib::extra::resman::mods::scan_priority_dir(
            &dir,
            &priority,
            &FallbackPolicy::default(),
        ))
        .unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn scan_priority_dir_missing_dir_empty() {
    let home = TempDir::new("rm_scan_priority_dir_missing");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("nope");

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let priority = Priority::parse("local").unwrap();
    let list = rt
        .block_on(rmlib::extra::resman::mods::scan_priority_dir(
            &dir,
            &priority,
            &FallbackPolicy::default(),
        ))
        .unwrap();
    assert!(list.is_empty());
}
