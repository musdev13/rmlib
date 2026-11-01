#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_version_install(opts_json: *const c_char) -> i32 {
    catch_i32(|| {
        let v = parse_json_arg(opts_json)?;
        let version_id = get_str_field(&v, "version_id");
        let json_path = get_opt_pathbuf(&v, "json_path");
        let versions_path_option = get_opt_pathbuf(&v, "versions_path");
        let libs_path_option = get_opt_pathbuf(&v, "libs_path");
        let assets_path_option = get_opt_pathbuf(&v, "assets_path");
        let soft = get_bool_field(&v, "soft", true);
        let iclient = get_bool_field(&v, "iclient", true);
        let ilibs = get_bool_field(&v, "ilibs", true);
        let iassets = get_bool_field(&v, "iassets", true);
        let alibs = get_u32_field(&v, "alibs", 150) as usize;
        let aassets = get_u32_field(&v, "aassets", 350) as usize;

        block_on(crate::core::version::install::install(
            version_id,
            json_path,
            versions_path_option,
            libs_path_option,
            assets_path_option,
            soft,
            iclient,
            ilibs,
            iassets,
            alibs,
            aassets,
        ));
        Ok(())
    })
}
