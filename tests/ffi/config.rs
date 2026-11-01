use super::helpers::*;
use crate::common::{HomeGuard, TempDir};
use rmlib::ffi::exports::*;

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
