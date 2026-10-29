#![cfg(feature = "ffi")]

mod common;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use common::{HomeGuard, TempDir};
use rmlib::ffi::exports::{
    rmlib_available_modules, rmlib_build_maven_path, rmlib_client_id, rmlib_client_token,
    rmlib_get_auth_path, rmlib_ms_auth_clear, rmlib_ms_auth_clear_at, rmlib_ms_auth_refresh,
    rmlib_ms_auth_refresh_at, rmlib_name, rmlib_paths_load, rmlib_version,
    rmlib_version_sort_by_type, rmlib_version_strip_version,
};
use rmlib::ffi::{rmlib_last_error, rmlib_string_free};

unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { rmlib_string_free(ptr) };
    Some(s)
}

unsafe fn take_error() -> Option<String> {
    let ptr = unsafe { rmlib_last_error() };
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { rmlib_string_free(ptr) };
    Some(s)
}

unsafe fn clear_error() {
    let _ = unsafe { take_error() };
}

#[test]
fn ffi_version_matches_crate_version() {
    let s = unsafe { take_string(rmlib_version()) }.unwrap();
    assert_eq!(s, rmlib::VERSION);
}

#[test]
fn ffi_name_matches_crate_name() {
    let s = unsafe { take_string(rmlib_name()) }.unwrap();
    assert_eq!(s, rmlib::NAME);
}

#[test]
fn ffi_client_id_matches_crate() {
    let s = unsafe { take_string(rmlib_client_id()) }.unwrap();
    assert_eq!(s, rmlib::CLIENT_ID);
}

#[test]
fn ffi_client_token_matches_crate() {
    let s = unsafe { take_string(rmlib_client_token()) }.unwrap();
    assert_eq!(s, rmlib::CLIENT_TOKEN);
}

#[test]
fn ffi_available_modules_is_valid_json() {
    let s = unsafe { take_string(rmlib_available_modules()) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let arr = v.as_array().expect("expected JSON array");
    assert!(arr.iter().any(|m| m["name"] == "core"));
}

#[test]
fn ffi_build_maven_path_valid() {
    let name = CString::new("net.minecraft:client:1.20.1").unwrap();
    let s =
        unsafe { take_string(rmlib_build_maven_path(name.as_ptr(), std::ptr::null())) }.unwrap();
    assert_eq!(s, "net/minecraft/client/1.20.1/client-1.20.1.jar");
}

#[test]
fn ffi_build_maven_path_with_classifier() {
    let name = CString::new("net.minecraft:client:1.20.1").unwrap();
    let classifier = CString::new("natives-linux").unwrap();
    let s =
        unsafe { take_string(rmlib_build_maven_path(name.as_ptr(), classifier.as_ptr())) }.unwrap();
    assert_eq!(
        s,
        "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
    );
}

#[test]
fn ffi_build_maven_path_invalid_returns_null_and_sets_error() {
    unsafe { clear_error() };
    let name = CString::new("bad").unwrap();
    let ptr = unsafe { rmlib_build_maven_path(name.as_ptr(), std::ptr::null()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("expected last_error to be set");
    assert!(err.contains("invalid maven"));
}

#[test]
fn ffi_last_error_is_one_shot() {
    let name = CString::new("bad").unwrap();
    let _ = unsafe { rmlib_build_maven_path(name.as_ptr(), std::ptr::null()) };
    let first = unsafe { take_error() };
    assert!(first.is_some());
    let second = unsafe { take_error() };
    assert!(second.is_none(), "last_error should be cleared after read");
}

#[test]
fn ffi_sort_by_type_release() {
    let input = r#"[{"id":"a","type":"release"},{"id":"b","type":"snapshot"}]"#;
    let json = CString::new(input).unwrap();
    let s = unsafe { take_string(rmlib_version_sort_by_type(json.as_ptr(), 0)) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "a");
}

#[test]
fn ffi_sort_by_type_invalid_type_returns_null() {
    unsafe { clear_error() };
    let input = CString::new("[]").unwrap();
    let ptr = unsafe { rmlib_version_sort_by_type(input.as_ptr(), 42) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("expected error");
    assert!(err.contains("invalid VersionType"));
}

#[test]
fn ffi_sort_by_type_invalid_json_returns_null() {
    unsafe { clear_error() };
    let input = CString::new("not json").unwrap();
    let ptr = unsafe { rmlib_version_sort_by_type(input.as_ptr(), 0) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("expected error");
    assert!(err.contains("invalid json"));
}

#[test]
fn ffi_strip_version_keeps_id() {
    let input = r#"[{"id":"a","type":"release","url":"u"}]"#;
    let json = CString::new(input).unwrap();
    let s = unsafe {
        take_string(rmlib_version_strip_version(
            json.as_ptr(),
            true,
            false,
            false,
            false,
            false,
        ))
    }
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v[0], serde_json::json!({"id": "a"}));
}

#[test]
fn ffi_strip_version_keeps_all() {
    let input = r#"[{"id":"a","type":"release","url":"u","releaseTime":"r","time":"t"}]"#;
    let json = CString::new(input).unwrap();
    let s = unsafe {
        take_string(rmlib_version_strip_version(
            json.as_ptr(),
            true,
            true,
            true,
            true,
            true,
        ))
    }
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v[0]["id"], "a");
    assert_eq!(v[0]["type"], "release");
    assert_eq!(v[0]["url"], "u");
}

#[test]
fn ffi_string_free_null_is_noop() {
    unsafe { rmlib_string_free(std::ptr::null_mut()) };
}

#[test]
fn ffi_get_auth_path_returns_default() {
    let home = TempDir::new("ffi_auth_path");
    let _guard = HomeGuard::new(home.path());

    let s = unsafe { take_string(rmlib_get_auth_path()) }.unwrap();
    assert_eq!(s, "~/rml/auth");
}

#[test]
fn ffi_paths_load_includes_auth() {
    let home = TempDir::new("ffi_paths_auth");
    let _guard = HomeGuard::new(home.path());

    let s = unsafe { take_string(rmlib_paths_load()) }.unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["auth"], "~/rml/auth");
    assert_eq!(v["versions"], "~/rml/versions");
}

#[test]
fn ffi_ms_auth_clear_ok() {
    let home = TempDir::new("ffi_ms_clear");
    let _guard = HomeGuard::new(home.path());

    assert_eq!(unsafe { rmlib_ms_auth_clear() }, 0);
}

#[test]
fn ffi_ms_auth_refresh_without_tokens_errors() {
    let home = TempDir::new("ffi_ms_refresh_no_tokens");
    let _guard = HomeGuard::new(home.path());

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_ms_auth_refresh() };
    assert!(ptr.is_null(), "expected NULL without saved tokens");
    let err = unsafe { take_error() }.expect("last_error should be set");
    assert!(
        err.contains("no saved ms tokens"),
        "unexpected error: {err}"
    );
}

#[test]
fn ffi_ms_auth_clear_at_custom_dir_ok() {
    let home = TempDir::new("ffi_ms_clear_at");
    let _guard = HomeGuard::new(home.path());

    let dir = home.join("custom_auth");
    let c = CString::new(dir.to_string_lossy().to_string()).unwrap();

    rmlib::core::auth::save_tokens_at(
        Some(&dir),
        &rmlib::core::auth::MsTokens {
            access_token: "a".to_string(),
            refresh_token: "r".to_string(),
            expires_in: None,
        },
    )
    .unwrap();
    assert!(dir.join("ms.json").exists());

    let rc = unsafe { rmlib_ms_auth_clear_at(c.as_ptr()) };
    assert_eq!(rc, 0);
    assert!(!dir.join("ms.json").exists());
}

#[test]
fn ffi_ms_auth_clear_at_null_uses_config() {
    let home = TempDir::new("ffi_ms_clear_at_null");
    let _guard = HomeGuard::new(home.path());

    let rc = unsafe { rmlib_ms_auth_clear_at(std::ptr::null()) };
    assert_eq!(rc, 0);
}

#[test]
fn ffi_ms_auth_refresh_at_null_errors_like_default() {
    let home = TempDir::new("ffi_ms_refresh_at_null");
    let _guard = HomeGuard::new(home.path());

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_ms_auth_refresh_at(std::ptr::null()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error should be set");
    assert!(
        err.contains("no saved ms tokens"),
        "unexpected error: {err}"
    );
}

#[test]
fn ffi_ms_auth_refresh_at_custom_dir_reports_no_tokens() {
    let home = TempDir::new("ffi_ms_refresh_at_custom");
    let _guard = HomeGuard::new(home.path());

    let custom = home.join("empty_auth");
    let c = CString::new(custom.to_string_lossy().to_string()).unwrap();

    unsafe { clear_error() };
    let ptr = unsafe { rmlib_ms_auth_refresh_at(c.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error should be set");
    assert!(
        err.contains("no saved ms tokens"),
        "unexpected error: {err}"
    );
}
