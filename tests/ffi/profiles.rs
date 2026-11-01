use super::helpers::*;
use crate::common::{HomeGuard, TempDir};
use rmlib::ffi::exports::*;
use std::ffi::CString;

#[test]
fn ffi_profile_save_load_roundtrip() {
    let home = TempDir::new("ffi_prof_roundtrip");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ely").unwrap();
    let json = profile_json();

    let path = unsafe { take_string(rmlib_profile_save(kind.as_ptr(), json.as_ptr())) }
        .expect("save should return path");
    assert!(path.ends_with("Musek.json"));

    let user = CString::new("Musek").unwrap();
    let loaded = unsafe { take_string(rmlib_profile_load(kind.as_ptr(), user.as_ptr())) }
        .expect("load should return json");
    let v: serde_json::Value = serde_json::from_str(&loaded).unwrap();
    assert_eq!(v["access_token"], "tok");
    assert_eq!(v["username"], "Musek");
}

#[test]
fn ffi_profile_load_missing_sets_error() {
    let home = TempDir::new("ffi_prof_missing");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ms").unwrap();
    let user = CString::new("Ghost").unwrap();

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_profile_load(kind.as_ptr(), user.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("not found"), "unexpected error: {err}");
}

#[test]
fn ffi_profile_list_returns_sorted_array() {
    let home = TempDir::new("ffi_prof_list");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ms").unwrap();
    for name in ["bob", "alice"] {
        let json = CString::new(format!(r#"{{"username":"{name}","access_token":"t"}}"#)).unwrap();
        unsafe { take_string(rmlib_profile_save(kind.as_ptr(), json.as_ptr())) }.unwrap();
    }

    let raw = unsafe { take_string(rmlib_profile_list(kind.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v, serde_json::json!(["alice", "bob"]));
}

#[test]
fn ffi_profile_delete_at_custom_dir() {
    let home = TempDir::new("ffi_prof_delete_at");
    let _guard = HomeGuard::new(home.path());

    let custom = home.join("custom_auth");
    let dir = CString::new(custom.to_string_lossy().to_string()).unwrap();
    let kind = CString::new("ely").unwrap();
    let json = profile_json();
    let user = CString::new("Musek").unwrap();

    unsafe {
        take_string(rmlib_profile_save_at(
            dir.as_ptr(),
            kind.as_ptr(),
            json.as_ptr(),
        ))
    }
    .unwrap();
    assert!(
        custom
            .join("profiles")
            .join("ely")
            .join("Musek.json")
            .exists()
    );

    let rc = unsafe { rmlib_profile_delete_at(dir.as_ptr(), kind.as_ptr(), user.as_ptr()) };
    assert_eq!(rc, 0);
    assert!(
        !custom
            .join("profiles")
            .join("ely")
            .join("Musek.json")
            .exists()
    );
}

#[test]
fn ffi_profile_delete_missing_is_ok() {
    let home = TempDir::new("ffi_prof_delete_missing");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ms").unwrap();
    let user = CString::new("Ghost").unwrap();
    let rc = unsafe { rmlib_profile_delete(kind.as_ptr(), user.as_ptr()) };
    assert_eq!(rc, 0);
}

#[test]
fn ffi_profile_invalid_kind_errors() {
    let home = TempDir::new("ffi_prof_bad_kind");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("steam").unwrap();
    let user = CString::new("x").unwrap();

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_profile_load(kind.as_ptr(), user.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("invalid profile kind"), "unexpected: {err}");
}

#[test]
fn ffi_profile_invalid_json_errors() {
    let home = TempDir::new("ffi_prof_bad_json");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ely").unwrap();
    let bad = CString::new("not json").unwrap();

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_profile_save(kind.as_ptr(), bad.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("invalid profile json"), "unexpected: {err}");
}

#[test]
fn ffi_profile_path_returns_layout() {
    let home = TempDir::new("ffi_prof_path");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ely").unwrap();
    let user = CString::new("Musek").unwrap();
    let path = unsafe { take_string(rmlib_profile_path(kind.as_ptr(), user.as_ptr())) }.unwrap();
    assert!(path.ends_with("profiles/ely/Musek.json"), "got: {path}");
}

#[test]
fn ffi_profile_list_at_null_uses_config() {
    let home = TempDir::new("ffi_prof_list_null");
    let _guard = HomeGuard::new(home.path());

    let kind = CString::new("ms").unwrap();
    let raw =
        unsafe { take_string(rmlib_profile_list_at(std::ptr::null(), kind.as_ptr())) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v, serde_json::json!([]));
}

#[test]
fn ffi_profile_load_at_custom_dir() {
    let home = TempDir::new("ffi_prof_load_at");
    let _guard = HomeGuard::new(home.path());

    let custom = home.join("custom_auth");
    let dir = CString::new(custom.to_string_lossy().to_string()).unwrap();
    let kind = CString::new("ms").unwrap();
    let user = CString::new("Musek").unwrap();
    let json = CString::new(r#"{"username":"Musek","access_token":"at-tok"}"#).unwrap();

    unsafe {
        take_string(rmlib_profile_save_at(
            dir.as_ptr(),
            kind.as_ptr(),
            json.as_ptr(),
        ))
    }
    .unwrap();
    let loaded = unsafe {
        take_string(rmlib_profile_load_at(
            dir.as_ptr(),
            kind.as_ptr(),
            user.as_ptr(),
        ))
    }
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&loaded).unwrap();
    assert_eq!(v["access_token"], "at-tok");
}
