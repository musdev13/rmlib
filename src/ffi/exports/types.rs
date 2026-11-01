#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_build_maven_path(
    name: *const c_char,
    classifier: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let name = unsafe { cstr_to_string(name) }?;
        let classifier = unsafe { opt_cstr_to_string(classifier) }?;
        match crate::core::types::build_maven_path(&name, classifier.as_deref()) {
            Some(p) => Ok(string_to_c(p)),
            None => Err("invalid maven coordinate".to_string()),
        }
    })
}
