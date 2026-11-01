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

fn rp(path: &Path, fmt: u32) {
    let meta = format!(r#"{{"pack":{{"pack_format":{fmt},"description":"p"}}}}"#);
    write_zip(path, &[("pack.mcmeta", meta.as_bytes())]);
}

#[test]
fn ffi_resourcepacks_scan_and_info() {
    let home = TempDir::new("ffi_rp_scan");
    let _guard = HomeGuard::new(home.path());
    let game = home.join("game");
    rp(&game.join("resourcepacks").join("a.zip"), 34);

    let gp = CString::new(game.to_string_lossy().to_string()).unwrap();
    let raw = unsafe { take_string(rmlib_resman_resourcepacks_scan(gp.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);

    let p = CString::new(
        game.join("resourcepacks")
            .join("a.zip")
            .to_string_lossy()
            .to_string(),
    )
    .unwrap();
    let raw = unsafe { take_string(rmlib_resman_resourcepacks_info(p.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["pack_format"], 34);
}

#[test]
fn ffi_resourcepacks_invalid_file_errors() {
    let home = TempDir::new("ffi_rp_bad");
    let _guard = HomeGuard::new(home.path());
    let p = home.join("nope.zip");
    let c = CString::new(p.to_string_lossy().to_string()).unwrap();

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_resman_resourcepacks_info(c.as_ptr()) };
    assert!(ptr.is_null());
    assert!(unsafe { take_error() }.is_some());
}

#[test]
fn ffi_resourcepacks_scan_as_plain() {
    let home = TempDir::new("ffi_rp_as");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("rp");
    rp(&dir.join("a.zip"), 34);

    let d = CString::new(dir.to_string_lossy().to_string()).unwrap();
    let fields = CString::new("pack_format").unwrap();
    let fmt = CString::new("plain").unwrap();
    let out = unsafe {
        take_string(rmlib_resman_resourcepacks_scan_as(
            d.as_ptr(),
            fields.as_ptr(),
            fmt.as_ptr(),
        ))
    }
    .unwrap();
    assert!(out.contains("pack_format"));
}

#[test]
fn ffi_resourcepacks_scan_priority_dir_works() {
    let home = TempDir::new("ffi_rp_scan_priority_dir");
    let _guard = HomeGuard::new(home.path());
    let dir = home.join("rp");
    rp(&dir.join("a.zip"), 34);

    let d = CString::new(dir.to_string_lossy().to_string()).unwrap();
    let prio = CString::new("local").unwrap();

    let raw = unsafe {
        take_string(rmlib_resman_resourcepacks_scan_priority_dir(
            d.as_ptr(),
            prio.as_ptr(),
            std::ptr::null(),
        ))
    }
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["pack_format"], 34);
}
