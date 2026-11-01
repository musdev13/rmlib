use super::helpers::*;
use rmlib::ffi::exports::*;
use rmlib::ffi::rmlib_string_free;
use std::ffi::CString;

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
fn ffi_last_error_is_one_shot() {
    let name = CString::new("bad").unwrap();
    let _ = unsafe { rmlib_build_maven_path(name.as_ptr(), std::ptr::null()) };
    let first = unsafe { take_error() };
    assert!(first.is_some());
    let second = unsafe { take_error() };
    assert!(second.is_none(), "last_error should be cleared after read");
}

#[test]
fn ffi_string_free_null_is_noop() {
    unsafe { rmlib_string_free(std::ptr::null_mut()) };
}
