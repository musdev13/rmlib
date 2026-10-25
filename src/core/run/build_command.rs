use crate::core::run::assemble_command::assemble_argv;
use crate::core::run::{
    build_arguments, build_classpath, build_launch_context, load_version_data, prepare_paths,
    resolve_environment,
};

pub struct BuiltCommand {
    pub argv: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
fn build(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> Result<BuiltCommand, String> {
    let paths = prepare_paths(
        version_id,
        versions_path,
        assets_path,
        libs_path,
        native_path,
        game_path,
    )?;

    let version_data =
        load_version_data(version_id, fabric_version, neoforge_version, &paths, logs)?;

    let environment = resolve_environment();

    let classpath = build_classpath(
        &version_data.version_json,
        version_data.modloader_json.as_ref(),
        &paths,
        &version_data.loader_libs_path,
        &environment,
    );

    let context = build_launch_context(
        version_id,
        username,
        uuid,
        xuid,
        token,
        ely,
        &version_data,
        &paths,
        &environment,
        &classpath,
    );

    let arguments = build_arguments(
        ram_count,
        ely,
        betafix,
        neoforge_version,
        &paths,
        &version_data,
        &context,
        &classpath,
        custom_jvm_args,
        custom_game_args,
        logs,
    );

    let argv = assemble_argv(&version_data.main_class, arguments, &context);
    Ok(BuiltCommand { argv })
}

#[allow(clippy::too_many_arguments)]
pub fn build_argv(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> Result<Vec<String>, String> {
    build(
        version_id,
        fabric_version,
        neoforge_version,
        ram_count,
        username,
        uuid,
        xuid,
        token,
        ely,
        betafix,
        versions_path,
        assets_path,
        libs_path,
        native_path,
        game_path,
        custom_jvm_args,
        custom_game_args,
        logs,
    )
    .map(|c| c.argv)
}

#[allow(clippy::too_many_arguments)]
pub fn build_command(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> Result<String, String> {
    build(
        version_id,
        fabric_version,
        neoforge_version,
        ram_count,
        username,
        uuid,
        xuid,
        token,
        ely,
        betafix,
        versions_path,
        assets_path,
        libs_path,
        native_path,
        game_path,
        custom_jvm_args,
        custom_game_args,
        logs,
    )
    .map(|c| {
        c.argv
            .into_iter()
            .map(|arg| {
                if arg.contains(' ') {
                    format!("\"{}\"", arg)
                } else {
                    arg
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    })
}
