use super::helpers::*;
use rmlib::ffi::exports::*;
use std::ffi::CString;

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
