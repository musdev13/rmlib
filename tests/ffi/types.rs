use super::helpers::*;
use rmlib::ffi::exports::*;
use std::ffi::CString;

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
