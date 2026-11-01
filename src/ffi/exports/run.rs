#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
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
