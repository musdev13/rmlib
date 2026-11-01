use super::helpers::*;
use crate::common::{HomeGuard, TempDir};
use std::ffi::CString;

#[cfg(feature = "modrinth")]
#[test]
fn ffi_modrinth_project_empty_id_errors() {
    use rmlib::ffi::exports::rmlib_modrinth_project;
    let home = TempDir::new("ffi_mr_empty_id");
    let _guard = HomeGuard::new(home.path());

    let empty = CString::new("").unwrap();
    unsafe { clear_error() };
    let ptr = unsafe { rmlib_modrinth_project(empty.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("empty"), "unexpected: {err}");
}

#[cfg(feature = "modrinth")]
#[test]
fn ffi_modrinth_search_invalid_json_errors() {
    use rmlib::ffi::exports::rmlib_modrinth_search;
    let home = TempDir::new("ffi_mr_bad_json");
    let _guard = HomeGuard::new(home.path());

    let bad = CString::new("not json").unwrap();
    unsafe { clear_error() };
    let ptr = unsafe { rmlib_modrinth_search(bad.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("invalid"), "unexpected: {err}");
}

#[cfg(feature = "modrinth")]
#[test]
fn ffi_modrinth_version_empty_id_errors() {
    use rmlib::ffi::exports::rmlib_modrinth_version;
    let home = TempDir::new("ffi_mr_empty_ver");
    let _guard = HomeGuard::new(home.path());

    let empty = CString::new("").unwrap();
    unsafe { clear_error() };
    let ptr = unsafe { rmlib_modrinth_version(empty.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("empty"), "unexpected: {err}");
}

#[cfg(feature = "modrinth")]
#[test]
fn ffi_modrinth_install_invalid_json_errors() {
    use rmlib::ffi::exports::rmlib_modrinth_install;
    let home = TempDir::new("ffi_mr_install_bad");
    let _guard = HomeGuard::new(home.path());

    let id = CString::new("sodium").unwrap();
    let bad = CString::new("{bad}").unwrap();
    unsafe { clear_error() };
    let ptr = unsafe { rmlib_modrinth_install(id.as_ptr(), bad.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("invalid"), "unexpected: {err}");
}

#[cfg(feature = "modrinth")]
#[test]
fn ffi_modrinth_user_empty_errors() {
    use rmlib::ffi::exports::rmlib_modrinth_user;
    let home = TempDir::new("ffi_mr_user_empty");
    let _guard = HomeGuard::new(home.path());

    let empty = CString::new("").unwrap();
    unsafe { clear_error() };
    let ptr = unsafe { rmlib_modrinth_user(empty.as_ptr()) };
    assert!(ptr.is_null());
    let err = unsafe { take_error() }.expect("last_error");
    assert!(err.contains("empty"), "unexpected: {err}");
}
