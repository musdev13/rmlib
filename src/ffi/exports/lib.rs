#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version() -> *mut c_char {
    string_to_c(crate::VERSION.to_string())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_name() -> *mut c_char {
    string_to_c(crate::NAME.to_string())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_client_token() -> *mut c_char {
    string_to_c(crate::CLIENT_TOKEN.to_string())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_client_id() -> *mut c_char {
    string_to_c(crate::CLIENT_ID.to_string())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_available_modules() -> *mut c_char {
    catch_cstr(|| {
        let mods = crate::available_modules();
        let v: Vec<Value> = mods
            .iter()
            .map(|m| serde_json::json!({ "name": m.name }))
            .collect();
        json_out(&v)
    })
}
