use crate::core::run::build_classpath::ClasspathData;
use crate::core::run::dto::LaunchContext;
use crate::core::run::load_version_data::VersionData;
use crate::core::run::prepare_paths::LaunchPaths;
use crate::core::run::parse_argument_list::parse_argument_list;

pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}

pub fn build_arguments(
    ram_count: &str,
    ely: bool,
    betafix: bool,
    neoforge_version: Option<&str>,
    paths: &LaunchPaths,
    version_data: &VersionData,
    context: &LaunchContext,
    classpath: &ClasspathData,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> LaunchArguments {
    let mut jvm_args = Vec::new();

    jvm_args.push(format!("-Xmx{}", ram_count));

    jvm_args.push(format!(
        "-Djava.library.path={}",
        paths.native.display()
    ));

    if ely {
        if !betafix {
            let injector_path = paths.libs.join("authlib-injector.jar");

            if injector_path.exists() {
                jvm_args.push(format!(
                    "-javaagent:{}=ely.by",
                    injector_path.to_string_lossy()
                ));
            } else {
                musutils::log!(
                    logs,
                    "{}: Ely.by requested but authlib-injector.jar not found at {:?}",
                    musutils::types::Status::Warn.as_colored_str(),
                    injector_path
                );
            }
        } else {
            let proxy_port = 43523;

            jvm_args.push("-Dhttp.proxyHost=127.0.0.1".to_string());

            jvm_args.push(format!("-Dhttp.proxyPort={}", proxy_port));

            jvm_args.push(
                "-Dhttp.nonProxyHosts=localhost|127.0.0.1".to_string(),
            );
        }
    } else if betafix {
        jvm_args.push("-Dhttp.proxyHost=betacraft.uk".to_string());
    }

    let mut game_args = Vec::new();

    if let Some(arguments_obj) = version_data.version_json.get("arguments") {
        if let Some(jvm_value) = arguments_obj.get("jvm") {
            parse_argument_list(
                jvm_value,
                context,
                &mut jvm_args,
            );
        }

        if let Some(game_value) = arguments_obj.get("game") {
            parse_argument_list(
                game_value,
                context,
                &mut game_args,
            );
        }
    } else if let Some(minecraft_args_str) = version_data
        .version_json
        .get("minecraftArguments")
        .and_then(|value| value.as_str())
    {
        game_args.extend(
            minecraft_args_str
                .split_whitespace()
                .map(str::to_string),
        );
    }

    if let Some(loader_json) = version_data.modloader_json.as_ref() {
        if let Some(loader_args) = loader_json.get("arguments") {
            if let Some(jvm_value) = loader_args.get("jvm") {
                parse_argument_list(
                    jvm_value,
                    context,
                    &mut jvm_args,
                );
            }

            if let Some(game_value) = loader_args.get("game") {
                parse_argument_list(
                    game_value,
                    context,
                    &mut game_args,
                );
            }
        }
    }

    if neoforge_version.is_some() {
        let has_module_path =
            jvm_args.iter().enumerate().any(|(index, arg)| {
                if arg == "-p" || arg == "--module-path" {
                    index + 1 < jvm_args.len()
                } else {
                    false
                }
            });

        let has_classpath =
            jvm_args.iter().enumerate().any(|(index, arg)| {
                if arg == "-cp" || arg == "-classpath" {
                    index + 1 < jvm_args.len()
                } else {
                    arg.starts_with("-classpath")
                }
            });

        if !has_module_path && !has_classpath {
            jvm_args.push("-cp".to_string());
            jvm_args.push(classpath.classpath.clone());
        }
    } else if !jvm_args.contains(&"-cp".to_string())
        && !jvm_args.iter().any(|arg| {
            arg == "-classpath"
                || arg.starts_with("-classpath")
        })
    {
        jvm_args.push("-cp".to_string());
        jvm_args.push(classpath.classpath.clone());
    }

    jvm_args.extend(custom_jvm_args.iter().cloned());
    game_args.extend(custom_game_args.iter().cloned());

    LaunchArguments {
        jvm_args,
        game_args,
    }
}
