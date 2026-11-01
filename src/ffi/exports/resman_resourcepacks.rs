#![allow(clippy::missing_safety_doc)]

use super::helpers::*;

use crate::extra::resman::resourcepacks as rp;

fn parse_priority(s: Option<String>) -> Result<rp::Priority, String> {
    match s {
        Some(p) => rp::Priority::parse(&p).map_err(|e| e.to_string()),
        None => Ok(rp::Priority::default()),
    }
}

fn parse_fallback(json: Option<String>) -> Result<rp::FallbackPolicy, String> {
    let Some(raw) = json else {
        return Ok(rp::FallbackPolicy::default());
    };
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("invalid fallback json: {e}"))?;
    let policy = v
        .get("policy")
        .and_then(|p| p.as_str())
        .unwrap_or("placeholder");
    match policy {
        "placeholder" => Ok(rp::FallbackPolicy::Placeholder),
        "skip" => Ok(rp::FallbackPolicy::Skip),
        "fields" => {
            let fields: Vec<rp::Field> = v
                .get("fields")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str())
                        .filter_map(|s| rp::Field::parse(s).ok())
                        .collect()
                })
                .unwrap_or_default();
            Ok(rp::FallbackPolicy::Fields(fields))
        }
        other => Err(format!("unknown fallback policy {other:?}")),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_scan(game_path: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let gp = unsafe { cstr_to_string(game_path) }?;
        let list = rp::scan(std::path::Path::new(&gp)).map_err(|e| e.to_string())?;
        json_out(&list)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_scan_dir(dir: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let d = unsafe { cstr_to_string(dir) }?;
        let list = rp::scan_dir(std::path::Path::new(&d)).map_err(|e| e.to_string())?;
        json_out(&list)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_scan_priority(
    game_path: *const c_char,
    priority: *const c_char,
    fallback_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let gp = unsafe { cstr_to_string(game_path) }?;
        let pr = parse_priority(unsafe { opt_cstr_to_string(priority) }?)?;
        let fb = parse_fallback(unsafe { opt_cstr_to_string(fallback_json) }?)?;
        let list = block_on(rp::scan_priority(std::path::Path::new(&gp), &pr, &fb))
            .map_err(|e| e.to_string())?;
        json_out(&list)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_scan_priority_dir(
    dir: *const c_char,
    priority: *const c_char,
    fallback_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let d = unsafe { cstr_to_string(dir) }?;
        let pr = parse_priority(unsafe { opt_cstr_to_string(priority) }?)?;
        let fb = parse_fallback(unsafe { opt_cstr_to_string(fallback_json) }?)?;
        let list = block_on(rp::scan_priority_dir(std::path::Path::new(&d), &pr, &fb))
            .map_err(|e| e.to_string())?;
        json_out(&list)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_info(path: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let p = unsafe { cstr_to_string(path) }?;
        let info = rp::info_from_file(std::path::Path::new(&p)).map_err(|e| e.to_string())?;
        json_out(&info)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_info_priority(
    path: *const c_char,
    priority: *const c_char,
    fallback_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let p = unsafe { cstr_to_string(path) }?;
        let pr = parse_priority(unsafe { opt_cstr_to_string(priority) }?)?;
        let fb = parse_fallback(unsafe { opt_cstr_to_string(fallback_json) }?)?;
        match block_on(rp::info_priority(std::path::Path::new(&p), &pr, &fb))
            .map_err(|e| e.to_string())?
        {
            Some(info) => json_out(&info),
            None => Err("skipped by fallback policy".to_string()),
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_fields(
    path: *const c_char,
    fields: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let p = unsafe { cstr_to_string(path) }?;
        let f = unsafe { cstr_to_string(fields) }?;
        let list = rp::parse_fields(&f).map_err(|e| e.to_string())?;
        let map =
            rp::info_file_fields(std::path::Path::new(&p), &list).map_err(|e| e.to_string())?;
        json_out(&map)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_info_as(
    path: *const c_char,
    fields: *const c_char,
    format: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let p = unsafe { cstr_to_string(path) }?;
        let fmt_raw = unsafe { cstr_to_string(format) }?;
        let fmt = rp::Format::parse(&fmt_raw).map_err(|e| e.to_string())?;
        let fields_parsed = match unsafe { opt_cstr_to_string(fields) }? {
            Some(s) if !s.is_empty() => Some(rp::parse_fields(&s).map_err(|e| e.to_string())?),
            _ => None,
        };
        let out = rp::info_file_as(std::path::Path::new(&p), fields_parsed.as_deref(), fmt)
            .map_err(|e| e.to_string())?;
        Ok(string_to_c(out))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_resman_resourcepacks_scan_as(
    dir: *const c_char,
    fields: *const c_char,
    format: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let d = unsafe { cstr_to_string(dir) }?;
        let fmt_raw = unsafe { cstr_to_string(format) }?;
        let fmt = rp::Format::parse(&fmt_raw).map_err(|e| e.to_string())?;
        let fields_parsed = match unsafe { opt_cstr_to_string(fields) }? {
            Some(s) if !s.is_empty() => Some(rp::parse_fields(&s).map_err(|e| e.to_string())?),
            _ => None,
        };
        let out = rp::scan_as(std::path::Path::new(&d), fields_parsed.as_deref(), fmt)
            .map_err(|e| e.to_string())?;
        Ok(string_to_c(out))
    })
}
