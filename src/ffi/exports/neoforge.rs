#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "neoforge")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_neoforge_install(opts_json: *const c_char) -> i32 {
    catch_i32(|| {
        let v = parse_json_arg(opts_json)?;
        let version_id = get_str_field(&v, "version_id").ok_or("missing version_id")?;
        let neoforge_version =
            get_str_field(&v, "neoforge_version").ok_or("missing neoforge_version")?;
        let versions_path_option = get_opt_pathbuf(&v, "versions_path");
        let libs_path_option = get_opt_pathbuf(&v, "libs_path");
        let assets_path_option = get_opt_pathbuf(&v, "assets_path");
        block_on(crate::extra::neoforge::install(
            version_id,
            neoforge_version,
            versions_path_option,
            libs_path_option,
            assets_path_option,
        ));
        Ok(())
    })
}

#[cfg(feature = "neoforge")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_neoforge_fetch_list(version_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let list = block_on(crate::extra::neoforge::fetch_list(&id));
        json_out(&list)
    })
}
