use super::helpers::*;
use crate::common::{HomeGuard, TempDir};
use rmlib::ffi::exports::*;
use std::ffi::CString;

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
