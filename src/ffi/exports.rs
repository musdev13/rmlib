#![allow(clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::os::raw::c_char;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

use serde_json::Value;

use super::error::{panic_msg, set_last_error};
use super::runtime::block_on;
use super::strings::{cstr_to_string, opt_cstr_to_string, string_to_c};

fn catch_cstr<F: FnOnce() -> Result<*mut c_char, String>>(f: F) -> *mut c_char {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
        Err(p) => {
            set_last_error(panic_msg(&*p));
            std::ptr::null_mut()
        }
    }
}

fn catch_i32<F: FnOnce() -> Result<(), String>>(f: F) -> i32 {
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            set_last_error(e);
            -1
        }
        Err(p) => {
            set_last_error(panic_msg(&*p));
            -1
        }
    }
}

fn get_str_field(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string())
}

fn get_bool_field(v: &Value, k: &str, default: bool) -> bool {
    v.get(k).and_then(|x| x.as_bool()).unwrap_or(default)
}

fn get_u32_field(v: &Value, k: &str, default: u32) -> u32 {
    v.get(k)
        .and_then(|x| x.as_u64())
        .map(|n| n as u32)
        .unwrap_or(default)
}

fn get_string_list_field(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn get_opt_pathbuf(v: &Value, k: &str) -> Option<PathBuf> {
    get_str_field(v, k).map(PathBuf::from)
}

fn parse_json_arg(ptr: *const c_char) -> Result<Value, String> {
    let s = unsafe { cstr_to_string(ptr) }?;
    serde_json::from_str(&s).map_err(|e| format!("invalid json: {e}"))
}

fn json_out<T: serde::Serialize>(value: &T) -> Result<*mut c_char, String> {
    let s = serde_json::to_string(value).map_err(|e| format!("json encode: {e}"))?;
    Ok(string_to_c(s))
}

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

#[cfg(feature = "fabric")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_fabric_install(opts_json: *const c_char) -> i32 {
    catch_i32(|| {
        let v = parse_json_arg(opts_json)?;
        let version_id = get_str_field(&v, "version_id").ok_or("missing version_id")?;
        let fabric_version = get_str_field(&v, "fabric_version").ok_or("missing fabric_version")?;
        let versions_path_option = get_opt_pathbuf(&v, "versions_path");
        let libs_path_option = get_opt_pathbuf(&v, "libs_path");
        let assets_path_option = get_opt_pathbuf(&v, "assets_path");
        block_on(crate::extra::fabric::install(
            version_id,
            fabric_version,
            versions_path_option,
            libs_path_option,
            assets_path_option,
        ));
        Ok(())
    })
}

#[cfg(feature = "fabric")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_fabric_fetch_list(version_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let list = block_on(crate::extra::fabric::fetch_list(&id));
        json_out(&list)
    })
}

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

#[cfg(feature = "core")]
struct RunOptions {
    version_id: String,
    fabric_version: Option<String>,
    neoforge_version: Option<String>,
    ram_count: String,
    username: String,
    uuid: String,
    xuid: String,
    token: String,
    ely: bool,
    betafix: bool,
    versions_path: String,
    assets_path: String,
    libs_path: String,
    native_path: String,
    game_path: String,
    custom_jvm_args: Vec<String>,
    custom_game_args: Vec<String>,
    logs: bool,
}

#[cfg(feature = "core")]
fn parse_run_options(v: &Value) -> Result<RunOptions, String> {
    Ok(RunOptions {
        version_id: get_str_field(v, "version_id").ok_or("missing version_id")?,
        fabric_version: get_str_field(v, "fabric_version"),
        neoforge_version: get_str_field(v, "neoforge_version"),
        ram_count: get_str_field(v, "ram_count").unwrap_or_else(|| "2G".to_string()),
        username: get_str_field(v, "username").ok_or("missing username")?,
        uuid: get_str_field(v, "uuid").unwrap_or_default(),
        xuid: get_str_field(v, "xuid").unwrap_or_default(),
        token: get_str_field(v, "token").unwrap_or_default(),
        ely: get_bool_field(v, "ely", false),
        betafix: get_bool_field(v, "betafix", false),
        versions_path: get_str_field(v, "versions_path").ok_or("missing versions_path")?,
        assets_path: get_str_field(v, "assets_path").ok_or("missing assets_path")?,
        libs_path: get_str_field(v, "libs_path").ok_or("missing libs_path")?,
        native_path: get_str_field(v, "native_path").unwrap_or_default(),
        game_path: get_str_field(v, "game_path").unwrap_or_default(),
        custom_jvm_args: get_string_list_field(v, "custom_jvm_args"),
        custom_game_args: get_string_list_field(v, "custom_game_args"),
        logs: get_bool_field(v, "logs", false),
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_run_client(opts_json: *const c_char) -> i32 {
    catch_i32(|| {
        let v = parse_json_arg(opts_json)?;
        let o = parse_run_options(&v)?;
        let spawn_path = get_str_field(&v, "spawn_path").unwrap_or_default();
        let java_path = get_str_field(&v, "java_path");

        block_on(crate::core::run::run_client(
            &o.version_id,
            o.fabric_version.as_deref(),
            o.neoforge_version.as_deref(),
            &o.ram_count,
            &o.username,
            &o.uuid,
            &o.xuid,
            &o.token,
            o.ely,
            o.betafix,
            &o.versions_path,
            &o.assets_path,
            &o.libs_path,
            &o.native_path,
            &o.game_path,
            &spawn_path,
            java_path.as_deref(),
            &o.custom_jvm_args,
            &o.custom_game_args,
        ));
        Ok(())
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_build_argv(opts_json: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let v = parse_json_arg(opts_json)?;
        let o = parse_run_options(&v)?;
        let argv = crate::core::run::build_argv(
            &o.version_id,
            o.fabric_version.as_deref(),
            o.neoforge_version.as_deref(),
            &o.ram_count,
            &o.username,
            &o.uuid,
            &o.xuid,
            &o.token,
            o.ely,
            o.betafix,
            &o.versions_path,
            &o.assets_path,
            &o.libs_path,
            &o.native_path,
            &o.game_path,
            &o.custom_jvm_args,
            &o.custom_game_args,
            o.logs,
        )?;
        json_out(&argv)
    })
}

#[cfg(feature = "core")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_build_command(opts_json: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let v = parse_json_arg(opts_json)?;
        let o = parse_run_options(&v)?;
        let s = crate::core::run::build_command(
            &o.version_id,
            o.fabric_version.as_deref(),
            o.neoforge_version.as_deref(),
            &o.ram_count,
            &o.username,
            &o.uuid,
            &o.xuid,
            &o.token,
            o.ely,
            o.betafix,
            &o.versions_path,
            &o.assets_path,
            &o.libs_path,
            &o.native_path,
            &o.game_path,
            &o.custom_jvm_args,
            &o.custom_game_args,
            o.logs,
        )?;
        Ok(string_to_c(s))
    })
}

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

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_search(params_json: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        use crate::extra::modrinth::dto::SearchParams;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: SearchParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let query = crate::extra::modrinth::build_query(&params);
        let response = block_on(crate::extra::modrinth::client::get_json::<
            crate::extra::modrinth::dto::SearchResponse,
        >("/search", &query))
        .map_err(|e| e.to_string())?;
        json_out(&response)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_project(id_or_slug: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_slug) }?;
        let project = block_on(crate::extra::modrinth::project::get_project(&id))
            .map_err(|e| e.to_string())?;
        json_out(&project)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_project_by_hash(hash: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let h = unsafe { cstr_to_string(hash) }?;
        let project = block_on(crate::extra::modrinth::project::get_project_by_hash(&h))
            .map_err(|e| e.to_string())?;
        json_out(&project)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_project_versions(project_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(project_id) }?;
        let versions = block_on(crate::extra::modrinth::version::get_project_versions(
            &id,
            &crate::extra::modrinth::dto::VersionFilters::default(),
        ))
        .map_err(|e| e.to_string())?;
        json_out(&versions)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_version(version_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let v = block_on(crate::extra::modrinth::version::get_version(&id))
            .map_err(|e| e.to_string())?;
        json_out(&v)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_version_by_hash(hash: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let h = unsafe { cstr_to_string(hash) }?;
        let v = block_on(crate::extra::modrinth::version::get_version_by_hash(&h))
            .map_err(|e| e.to_string())?;
        json_out(&v)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_dependencies(project_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(project_id) }?;
        let deps = block_on(crate::extra::modrinth::project::get_project_dependencies(
            &id,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&deps)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_members(project_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(project_id) }?;
        let m = block_on(crate::extra::modrinth::project::get_project_members(&id))
            .map_err(|e| e.to_string())?;
        json_out(&m)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_user(id_or_username: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_username) }?;
        let u = block_on(crate::extra::modrinth::user::get_user(&id)).map_err(|e| e.to_string())?;
        json_out(&u)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_user_projects(user_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(user_id) }?;
        let p = block_on(crate::extra::modrinth::user::get_user_projects(&id))
            .map_err(|e| e.to_string())?;
        json_out(&p)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_team_members(team_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(team_id) }?;
        let m = block_on(crate::extra::modrinth::user::get_team_members(&id))
            .map_err(|e| e.to_string())?;
        json_out(&m)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_categories() -> *mut c_char {
    catch_cstr(|| {
        let c =
            block_on(crate::extra::modrinth::tag::get_categories()).map_err(|e| e.to_string())?;
        json_out(&c)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_loaders() -> *mut c_char {
    catch_cstr(|| {
        let l = block_on(crate::extra::modrinth::tag::get_loaders()).map_err(|e| e.to_string())?;
        json_out(&l)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_game_versions() -> *mut c_char {
    catch_cstr(|| {
        let g = block_on(crate::extra::modrinth::tag::get_game_versions())
            .map_err(|e| e.to_string())?;
        json_out(&g)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install(
    id_or_slug: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_slug) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_project(
            &id, &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install_version(
    version_id: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_version(
            &id, &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install_from_hash(
    hash: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let h = unsafe { cstr_to_string(hash) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_from_hash(
            &h, &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install_datapack(
    id_or_slug: *const c_char,
    world_name: *const c_char,
    game_path: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_slug) }?;
        let world = unsafe { cstr_to_string(world_name) }?;
        let gp = unsafe { cstr_to_string(game_path) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_datapack(
            &id,
            &world,
            std::path::Path::new(&gp),
            &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}
