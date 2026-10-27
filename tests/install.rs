#![cfg(feature = "core")]

mod common;

use std::fs;
use std::io::Write;
use std::path::Path;

use common::{HomeGuard, TempDir};
use rmlib::core::version::install::{
    extract_native_libraries, prepare_version_files, resolve_installation_paths,
};

fn native_ext() -> &'static str {
    if cfg!(target_os = "windows") {
        ".dll"
    } else if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    }
}

fn make_jar_with_native(path: &Path, native_name: &str, extra_meta: bool) {
    let file = fs::File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    if extra_meta {
        writer.start_file("META-INF/MANIFEST.MF", options).unwrap();
        writer.write_all(b"Manifest").unwrap();
    }

    writer.start_file(native_name, options).unwrap();
    writer.write_all(b"native-bytes").unwrap();

    writer.finish().unwrap();
}

#[test]
fn prepare_version_files_creates_dirs_and_writes_json() {
    let dir = TempDir::new("prep_files");
    let versions = dir.join("versions");
    let libs = dir.join("libs");
    let assets = dir.join("assets");
    let body = r#"{"id":"1.20.1","mainClass":"Main"}"#;

    let version_path = prepare_version_files(
        &versions,
        &libs,
        &assets,
        "1.20.1",
        body,
        "==========",
    );

    assert_eq!(version_path, versions.join("1.20.1"));
    assert!(versions.is_dir());
    assert!(libs.is_dir());
    assert!(assets.is_dir());
    assert!(version_path.is_dir());

    let written = fs::read_to_string(version_path.join("1.20.1.json")).unwrap();
    assert_eq!(written, body);
}

#[test]
fn prepare_version_files_is_idempotent() {
    let dir = TempDir::new("prep_files_idem");
    let versions = dir.join("versions");
    let libs = dir.join("libs");
    let assets = dir.join("assets");

    prepare_version_files(&versions, &libs, &assets, "1.20.1", "{}", "==");
    let p2 = prepare_version_files(&versions, &libs, &assets, "1.20.1", "{\"new\":1}", "==");

    let written = fs::read_to_string(p2.join("1.20.1.json")).unwrap();
    assert_eq!(written, "{\"new\":1}");
}

#[test]
fn resolve_installation_paths_all_some_flags_true() {
    let dir = TempDir::new("rip_some");
    let v = dir.join("v");
    let l = dir.join("l");
    let a = dir.join("a");

    let (cv, cl, ca, versions_path, libs_path, assets_path) =
        resolve_installation_paths(Some(v.clone()), Some(l.clone()), Some(a.clone()));

    assert!(cv);
    assert!(cl);
    assert!(ca);
    assert_eq!(versions_path, musutils::fs::expand(&v));
    assert_eq!(libs_path, musutils::fs::expand(&l));
    assert_eq!(assets_path, musutils::fs::expand(&a));
}

#[test]
fn resolve_installation_paths_all_none_flags_false() {
    let home = TempDir::new("rip_none_home");
    let _guard = HomeGuard::new(home.path());

    let (cv, cl, ca, versions_path, libs_path, assets_path) =
        resolve_installation_paths(None, None, None);

    assert!(!cv);
    assert!(!cl);
    assert!(!ca);
    assert!(versions_path.is_absolute());
    assert!(libs_path.is_absolute());
    assert!(assets_path.is_absolute());
}

#[test]
fn resolve_installation_paths_mixed_flags() {
    let home = TempDir::new("rip_mixed_home");
    let _guard = HomeGuard::new(home.path());
    let custom = home.join("custom_versions");

    let (cv, cl, ca, _, _, _) =
        resolve_installation_paths(Some(custom), None, None);
    assert!(cv);
    assert!(!cl);
    assert!(!ca);
}

#[test]
fn extract_native_libraries_extracts_os_specific_file() {
    let dir = TempDir::new("extract_ok");
    let libs = dir.join("libs");
    fs::create_dir_all(&libs).unwrap();

    let jar = libs.join("natives.jar");
    let native_name = format!("libfoo{}", native_ext());
    make_jar_with_native(&jar, &native_name, false);

    extract_native_libraries(&libs);

    let extracted = libs.join("natives").join(&native_name);
    assert!(extracted.exists(), "expected native {} to be extracted", extracted.display());
    assert_eq!(fs::read(&extracted).unwrap(), b"native-bytes");
}

#[test]
fn extract_native_libraries_skips_meta_inf() {
    let dir = TempDir::new("extract_meta");
    let libs = dir.join("libs");
    fs::create_dir_all(&libs).unwrap();

    let jar = libs.join("natives.jar");
    let native_name = format!("libfoo{}", native_ext());
    make_jar_with_native(&jar, &native_name, true);

    extract_native_libraries(&libs);

    let natives = libs.join("natives");
    let manifest = natives.join("MANIFEST.MF");
    assert!(!manifest.exists());
    assert!(natives.join(&native_name).exists());
}

#[test]
fn extract_native_libraries_ignores_non_native_jars() {
    let dir = TempDir::new("extract_no_native");
    let libs = dir.join("libs");
    fs::create_dir_all(&libs).unwrap();

    let jar = libs.join("plain.jar");
    let file = fs::File::create(&jar).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    writer.start_file("some/class.class", options).unwrap();
    writer.write_all(b"bytes").unwrap();
    writer.finish().unwrap();

    extract_native_libraries(&libs);

    let natives = libs.join("natives");
    let entries: Vec<_> = fs::read_dir(&natives)
        .map(|rd| rd.filter_map(|e| e.ok()).collect())
        .unwrap_or_default();
    assert!(entries.is_empty(), "expected no natives, got {:?}", entries.len());
}

#[test]
fn extract_native_libraries_empty_dir_ok() {
    let dir = TempDir::new("extract_empty");
    let libs = dir.join("libs");
    fs::create_dir_all(&libs).unwrap();

    extract_native_libraries(&libs);

    assert!(libs.join("natives").is_dir());
}
