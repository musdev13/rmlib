#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
fn auth_dir_arg(ptr: *const c_char) -> Result<Option<PathBuf>, String> {
    Ok(unsafe { opt_cstr_to_string(ptr) }?.map(PathBuf::from))
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_login_at(dir: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let dir = auth_dir_arg(dir)?;
        let tokens =
            block_on(crate::core::auth::login_at(dir.as_deref())).map_err(|e| e.to_string())?;
        json_out(&tokens)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_login() -> *mut c_char {
    unsafe { rmlib_ms_auth_login_at(std::ptr::null()) }
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_refresh_at(dir: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let dir = auth_dir_arg(dir)?;
        let saved = crate::core::auth::load_tokens_at(dir.as_deref()).map_err(|e| e.to_string())?;
        let refresh_token = saved
            .ok_or_else(|| "no saved ms tokens; run rmlib_ms_auth_login first".to_string())?
            .refresh_token;
        let tokens = block_on(crate::core::auth::refresh_at(
            dir.as_deref(),
            &refresh_token,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&tokens)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_refresh() -> *mut c_char {
    unsafe { rmlib_ms_auth_refresh_at(std::ptr::null()) }
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_get_data_at(dir: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let dir = auth_dir_arg(dir)?;
        let profile =
            block_on(crate::core::auth::get_data_at(dir.as_deref())).map_err(|e| e.to_string())?;
        json_out(&profile)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_get_data() -> *mut c_char {
    unsafe { rmlib_ms_auth_get_data_at(std::ptr::null()) }
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_clear_at(dir: *const c_char) -> i32 {
    catch_i32(|| {
        let dir = auth_dir_arg(dir)?;
        crate::core::auth::clear_tokens_at(dir.as_deref()).map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ms_auth_clear() -> i32 {
    unsafe { rmlib_ms_auth_clear_at(std::ptr::null()) }
}
