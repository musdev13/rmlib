#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version_fetch_list() -> *mut c_char {
    catch_cstr(|| {
        let v = block_on(crate::core::version::fetch_list());
        json_out(&v)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version_get_json(version_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let s = block_on(crate::core::version::get_version_json(id));
        Ok(string_to_c(s))
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version_get_json_url(version_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let s = block_on(crate::core::version::get_version_json_url(id));
        Ok(string_to_c(s))
    })
}

#[cfg(feature = "core")]
fn int_to_version_type(i: i32) -> Result<crate::core::types::VersionType, String> {
    match i {
        0 => Ok(crate::core::types::VersionType::Release),
        1 => Ok(crate::core::types::VersionType::Snapshot),
        2 => Ok(crate::core::types::VersionType::OldBeta),
        3 => Ok(crate::core::types::VersionType::OldAlpha),
        _ => Err(format!("invalid VersionType: {i}")),
    }
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version_sort_by_type(
    json: *const c_char,
    version_type: i32,
) -> *mut c_char {
    catch_cstr(|| {
        let s = unsafe { cstr_to_string(json) }?;
        let v: Vec<Value> = serde_json::from_str(&s).map_err(|e| format!("invalid json: {e}"))?;
        let vt = int_to_version_type(version_type)?;
        let res = crate::core::version::sort_by_type(&v, vt);
        json_out(&res)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version_strip_version(
    json: *const c_char,
    keep_id: bool,
    keep_release_time: bool,
    keep_time: bool,
    keep_type: bool,
    keep_url: bool,
) -> *mut c_char {
    catch_cstr(|| {
        let s = unsafe { cstr_to_string(json) }?;
        let v: Vec<Value> = serde_json::from_str(&s).map_err(|e| format!("invalid json: {e}"))?;
        let res = crate::core::version::strip_version(
            &v,
            keep_id,
            keep_release_time,
            keep_time,
            keep_type,
            keep_url,
        );
        json_out(&res)
    })
}
