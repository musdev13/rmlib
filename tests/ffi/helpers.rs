use std::ffi::CStr;
use std::os::raw::c_char;

use rmlib::ffi::{rmlib_last_error, rmlib_string_free};

pub unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { rmlib_string_free(ptr) };
    Some(s)
}

pub unsafe fn take_error() -> Option<String> {
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

pub unsafe fn clear_error() {
    let _ = unsafe { take_error() };
}

pub fn profile_json() -> std::ffi::CString {
    std::ffi::CString::new(
        r#"{"username":"Musek","uuid":"u","access_token":"tok","client_token":"ct"}"#,
    )
    .unwrap()
}
