#![allow(clippy::missing_safety_doc)]

pub(crate) use std::ffi::c_void;
pub(crate) use std::os::raw::c_char;
pub(crate) use std::panic::AssertUnwindSafe;
pub(crate) use std::path::PathBuf;

pub(crate) use serde_json::Value;

pub(crate) use super::super::error::{panic_msg, set_last_error};
pub(crate) use super::super::runtime::block_on;
pub(crate) use super::super::strings::{cstr_to_string, opt_cstr_to_string, string_to_c};

pub(crate) fn catch_cstr<F: FnOnce() -> Result<*mut c_char, String>>(f: F) -> *mut c_char {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
        Err(p) => {
            set_last_error(panic_msg(&*p));
            std::ptr::null_mut()
        }
    }
}

pub(crate) fn catch_i32<F: FnOnce() -> Result<(), String>>(f: F) -> i32 {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            set_last_error(e);
            -1
        }
        Err(p) => {
            set_last_error(panic_msg(&*p));
            -1
        }
    }
}

pub(crate) fn get_str_field(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string())
}

pub(crate) fn get_bool_field(v: &Value, k: &str, default: bool) -> bool {
    v.get(k).and_then(|x| x.as_bool()).unwrap_or(default)
}

pub(crate) fn get_u32_field(v: &Value, k: &str, default: u32) -> u32 {
    v.get(k)
        .and_then(|x| x.as_u64())
        .map(|n| n as u32)
        .unwrap_or(default)
}

pub(crate) fn get_string_list_field(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn get_opt_pathbuf(v: &Value, k: &str) -> Option<PathBuf> {
    get_str_field(v, k).map(PathBuf::from)
}

pub(crate) fn parse_json_arg(ptr: *const c_char) -> Result<Value, String> {
    let s = unsafe { cstr_to_string(ptr) }?;
    serde_json::from_str(&s).map_err(|e| format!("invalid json: {e}"))
}

pub(crate) fn json_out<T: serde::Serialize>(value: &T) -> Result<*mut c_char, String> {
    let s = serde_json::to_string(value).map_err(|e| format!("json encode: {e}"))?;
    Ok(string_to_c(s))
}
