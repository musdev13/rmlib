#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_config_init(force: bool) -> i32 {
    catch_i32(|| {
        crate::core::config::init(force);
        Ok(())
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_get_versions_path() -> *mut c_char {
    catch_cstr(|| Ok(string_to_c(crate::core::config::get_versions_path())))
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_get_libs_path() -> *mut c_char {
    catch_cstr(|| Ok(string_to_c(crate::core::config::get_libs_path())))
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_get_assets_path() -> *mut c_char {
    catch_cstr(|| Ok(string_to_c(crate::core::config::get_assets_path())))
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_get_auth_path() -> *mut c_char {
    catch_cstr(|| Ok(string_to_c(crate::core::config::get_auth_path())))
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_paths_load() -> *mut c_char {
    catch_cstr(|| {
        let p = crate::core::config::Paths::load().map_err(|e| e.to_string())?;
        let v = serde_json::json!({
            "versions": p.versions,
            "libs": p.libs,
            "assets": p.assets,
            "auth": p.auth,
        });
        json_out(&v)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_paths_init(force: bool) -> i32 {
    catch_i32(|| {
        crate::core::config::Paths::init(force).map_err(|e| e.to_string())?;
        Ok(())
    })
}
