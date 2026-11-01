#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "profiles")]
fn profile_kind_arg(s: &str) -> Result<crate::extra::profiles::ProfileKind, String> {
    match s {
        "ely" => Ok(crate::extra::profiles::ProfileKind::Ely),
        "ms" => Ok(crate::extra::profiles::ProfileKind::Ms),
        other => Err(format!(
            "invalid profile kind: {other:?} (expected \"ely\" or \"ms\")"
        )),
    }
}

#[cfg(feature = "profiles")]
fn profile_dir_arg(ptr: *const c_char) -> Result<Option<PathBuf>, String> {
    Ok(unsafe { opt_cstr_to_string(ptr) }?.map(PathBuf::from))
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_save_at(
    dir: *const c_char,
    kind: *const c_char,
    json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let dir = profile_dir_arg(dir)?;
        let kind = profile_kind_arg(&unsafe { cstr_to_string(kind) }?)?;
        let profile: crate::extra::profiles::Profile =
            serde_json::from_str(&unsafe { cstr_to_string(json) }?)
                .map_err(|e| format!("invalid profile json: {e}"))?;
        let path = crate::extra::profiles::save_profile_at(dir.as_deref(), kind, &profile)
            .map_err(|e| e.to_string())?;
        Ok(string_to_c(path.to_string_lossy().into_owned()))
    })
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_save(
    kind: *const c_char,
    json: *const c_char,
) -> *mut c_char {
    unsafe { rmlib_profile_save_at(std::ptr::null(), kind, json) }
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_load_at(
    dir: *const c_char,
    kind: *const c_char,
    username: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let dir = profile_dir_arg(dir)?;
        let kind = profile_kind_arg(&unsafe { cstr_to_string(kind) }?)?;
        let username = unsafe { cstr_to_string(username) }?;
        match crate::extra::profiles::load_profile_at(dir.as_deref(), kind, &username)
            .map_err(|e| e.to_string())?
        {
            Some(profile) => json_out(&profile),
            None => Err(format!("profile not found: {username}")),
        }
    })
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_load(
    kind: *const c_char,
    username: *const c_char,
) -> *mut c_char {
    unsafe { rmlib_profile_load_at(std::ptr::null(), kind, username) }
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_delete_at(
    dir: *const c_char,
    kind: *const c_char,
    username: *const c_char,
) -> i32 {
    catch_i32(|| {
        let dir = profile_dir_arg(dir)?;
        let kind = profile_kind_arg(&unsafe { cstr_to_string(kind) }?)?;
        let username = unsafe { cstr_to_string(username) }?;
        crate::extra::profiles::delete_profile_at(dir.as_deref(), kind, &username)
            .map_err(|e| e.to_string())
    })
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_delete(kind: *const c_char, username: *const c_char) -> i32 {
    unsafe { rmlib_profile_delete_at(std::ptr::null(), kind, username) }
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_list_at(
    dir: *const c_char,
    kind: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let dir = profile_dir_arg(dir)?;
        let kind = profile_kind_arg(&unsafe { cstr_to_string(kind) }?)?;
        let list = crate::extra::profiles::list_profiles_at(dir.as_deref(), kind)
            .map_err(|e| e.to_string())?;
        json_out(&list)
    })
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_list(kind: *const c_char) -> *mut c_char {
    unsafe { rmlib_profile_list_at(std::ptr::null(), kind) }
}

#[cfg(feature = "profiles")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_profile_path(
    kind: *const c_char,
    username: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let kind = profile_kind_arg(&unsafe { cstr_to_string(kind) }?)?;
        let username = unsafe { cstr_to_string(username) }?;
        let path =
            crate::extra::profiles::profile_path(kind, &username).map_err(|e| e.to_string())?;
        Ok(string_to_c(path.to_string_lossy().into_owned()))
    })
}
