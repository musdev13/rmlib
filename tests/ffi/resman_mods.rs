use super::helpers::*;
use std::ffi::CString;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::common::{HomeGuard, TempDir};
use rmlib::ffi::exports::*;

fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
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

fn fabric_jar(path: &Path, id: &str) {
    let fmj = format!(
        r#"{{"schemaVersion":1,"id":"{id}","version":"1.0.0","name":"{id}","description":"d","authors":["A"],"license":"MIT"}}"#
    );
    write_zip(path, &[("fabric.mod.json", fmj.as_bytes())]);
}

#[test]
fn ffi_mods_scan_and_info() {
    let home = TempDir::new("ffi_rm_scan");
    let _guard = HomeGuard::new(home.path());
    let game = home.join("game");
    fabric_jar(&game.join("mods").join("a.jar"), "alpha");
    fabric_jar(&game.join("mods").join("b.jar"), "beta");

    let gp = CString::new(game.to_string_lossy().to_string()).unwrap();
    let raw = unsafe { take_string(rmlib_resman_mods_scan(gp.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 2);

    let p = CString::new(
        game.join("mods")
            .join("a.jar")
            .to_string_lossy()
            .to_string(),
    )
    .unwrap();
    let raw = unsafe { take_string(rmlib_resman_mods_info(p.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["id"], "alpha");
}

#[test]
fn ffi_mods_toggle_and_is_enabled() {
    let home = TempDir::new("ffi_rm_toggle");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("m.jar");
    fabric_jar(&p, "m");
    let c = CString::new(p.to_string_lossy().to_string()).unwrap();

    assert_eq!(unsafe { rmlib_resman_mods_is_enabled(c.as_ptr()) }, 0);

    let raw = unsafe { take_string(rmlib_resman_mods_toggle(c.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["enabled"], false);

    let new_path = CString::new(v["path"].as_str().unwrap()).unwrap();
    assert_eq!(
        unsafe { rmlib_resman_mods_is_enabled(new_path.as_ptr()) },
        -1
    );
}

#[test]
fn ffi_mods_invalid_path_errors() {
    let home = TempDir::new("ffi_rm_bad");
    let _guard = HomeGuard::new(home.path());
    let p = CString::new(home.join("nope.jar").to_string_lossy().to_string()).unwrap();

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_resman_mods_info(p.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("not found"), "unexpected: {err}");
}

#[test]
fn ffi_mods_null_game_path_errors() {
    unsafe { clear_error() };
    let ptr = unsafe { rmlib_resman_mods_scan(std::ptr::null()) };
    assert!(ptr.is_null());
    assert!(unsafe { take_error() }.is_some());
}

#[test]
fn ffi_mods_scan_priority_dir_works() {
    let home = TempDir::new("ffi_rm_scan_priority_dir");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("mods");
    fabric_jar(&dir.join("a.jar"), "alpha");

    let d = CString::new(dir.to_string_lossy().to_string()).unwrap();
    let prio = CString::new("local").unwrap();

    let raw = unsafe {
        take_string(rmlib_resman_mods_scan_priority_dir(
            d.as_ptr(),
            prio.as_ptr(),
            std::ptr::null(),
        ))
    }
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["id"], "alpha");
}
