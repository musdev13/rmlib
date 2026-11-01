#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "ely")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ely_login(
    username: *const c_char,
    password: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let user = unsafe { cstr_to_string(username) }?;
        let pass = unsafe { cstr_to_string(password) }?;
        let res = block_on(crate::extra::ely::login(user, pass)).map_err(|e| e.to_string())?;
        json_out(&res)
    })
}

#[cfg(feature = "ely")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_ely_install_patch(opts_json: *const c_char) -> i32 {
    catch_i32(|| {
        let v = parse_json_arg(opts_json)?;
        let libs_path_option = get_opt_pathbuf(&v, "libs_path");
        let is_json = get_bool_field(&v, "is_json", false);
        block_on(crate::extra::ely::install_patch(libs_path_option, is_json))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(feature = "ely")]
#[unsafe(no_mangle)]
pub extern "C" fn rmlib_start_beta_proxy(port: u16) -> *mut c_void {
    match std::panic::catch_unwind(AssertUnwindSafe(|| {
        crate::extra::ely::start_beta_proxy(port)
    })) {
        Ok(Some(guard)) => Box::into_raw(Box::new(guard)) as *mut c_void,
        Ok(None) => {
            set_last_error("failed to start proxy (port unavailable?)");
            std::ptr::null_mut()
        }
        Err(p) => {
            set_last_error(panic_msg(&*p));
            std::ptr::null_mut()
        }
    }
}

#[cfg(feature = "ely")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_proxy_guard_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr as *mut crate::extra::ely::ProxyGuard));
        }
    }
}
