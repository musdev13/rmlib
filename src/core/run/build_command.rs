use std::collections::HashMap;
use std::fs;
use serde_json::Value;

use crate::core::run::dto::LaunchContext;
use crate::core::run::parse_argument_list::parse_argument_list;
use crate::core::run::replace_placeholders::replace_placeholders;
use crate::core::run::verify_libraries::verify_libraries;

pub fn build_command(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
) -> Result<String, String> {
    let norm_versions = musutils::fs::tilda_desir(versions_path);
    let norm_assets = musutils::fs::tilda_desir(assets_path);
    let norm_libs = musutils::fs::tilda_desir(libs_path);
    let norm_native = musutils::fs::tilda_desir(native_path);
    let norm_game = musutils::fs::tilda_desir(game_path);

    let json_path = norm_versions
        .join(version_id)
        .join(format!("{}.json", version_id));

    let client_jar_path = norm_versions
        .join(version_id)
        .join(format!("{}.jar", version_id));

    if !json_path.exists() {
        return Err(format!(
            "client json not found at {:?}",
            json_path
        ));
    }

    let json_content = fs::read_to_string(&json_path)
        .map_err(|e| format!("failed to read client json: {}", e))?;

    let version_json: Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("failed to parse client json: {}", e))?;

    let modloader_json: Option<Value> = if let Some(fab_ver) = fabric_version {
        let fabric_json_path = norm_versions
            .join("modloaders")
            .join("fabric")
            .join(version_id)
            .join(format!("{}.json", fab_ver));

        if fabric_json_path.exists() {
            println!(
                "{}: loading Fabric profile JSON from {:?}",
                musutils::types::Status::Inf.as_colored_str(),
                fabric_json_path
            );

            fs::read_to_string(&fabric_json_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        } else {
            println!(
                "{}: Fabric version `{}` requested, but profile JSON was not found at {:?}",
                musutils::types::Status::Warn.as_colored_str(),
                fab_ver,
                fabric_json_path
            );
            None
        }
    } else if let Some(neoforge_ver) = neoforge_version {
        let neoforge_json_path = norm_versions
            .join("modloaders")
            .join("neoforge")
            .join(version_id)
            .join(neoforge_ver)
            .join(format!("{}.json", neoforge_ver));

        if neoforge_json_path.exists() {
            println!(
                "{}: loading NeoForge profile JSON from {:?}",
                musutils::types::Status::Inf.as_colored_str(),
                neoforge_json_path
            );

            fs::read_to_string(&neoforge_json_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        } else {
            println!(
                "{}: NeoForge version `{}` requested, but profile JSON was not found at {:?}",
                musutils::types::Status::Warn.as_colored_str(),
                neoforge_ver,
                neoforge_json_path
            );
            None
        }
    } else {
        None
    };

    let my_os = match musutils::os::get_os() {
        musutils::os::OS::Windows => "windows",
        musutils::os::OS::Linux => "linux",
        musutils::os::OS::MacOs => "osx",
        _ => "unknown",
    }
    .to_string();

    let my_arch = match musutils::os::get_arch() {
        musutils::os::Arch::X64 => "64",
        musutils::os::Arch::X86 => "32",
        musutils::os::Arch::Arm64 => "arm64",
        musutils::os::Arch::Arm => "arm32",
        _ => "unknown",
    }
    .to_string();

    let main_class = modloader_json
        .as_ref()
        .and_then(|json| json.get("mainClass"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            version_json
                .get("mainClass")
                .and_then(|value| value.as_str())
        })
        .ok_or_else(|| "missing mainClass inside json".to_string())?
        .to_string();

    let asset_index_name = version_json
        .get("assetIndex")
        .and_then(|a| a.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("legacy")
        .to_string();

    let user_type = if ely { "mojang" } else { "msa" };

    let mut raw_libs = Vec::new();

    let loader_libs_path = if fabric_version.is_some() {
        norm_libs.join("modloaders").join("fabric")
    } else if neoforge_version.is_some() {
        norm_libs.join("modloaders").join("neoforge")
    } else {
        norm_libs.clone()
    };

    if let Some(ref loader_json) = modloader_json {
        let loader_libs = verify_libraries(
            loader_json,
            &loader_libs_path,
            &LaunchContext {
                my_os: my_os.clone(),
                my_arch: my_arch.clone(),
                placeholders: HashMap::new(),
            },
        );

        raw_libs.extend(loader_libs);
    }

    let vanilla_libs = verify_libraries(
        &version_json,
        &norm_libs,
        &LaunchContext {
            my_os: my_os.clone(),
            my_arch: my_arch.clone(),
            placeholders: HashMap::new(),
        },
    );

    raw_libs.extend(vanilla_libs);

    let mut seen_keys = std::collections::HashSet::new();
    let mut classpath_items = Vec::new();

    for lib in &raw_libs {
        if seen_keys.insert(lib.key.clone()) {
            classpath_items.push(lib.path.clone());
        }
    }

    if client_jar_path.exists() {
        classpath_items.push(client_jar_path.clone());
    }

    let classpath_separator = if cfg!(windows) { ";" } else { ":" };

    let final_classpath = classpath_items
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(classpath_separator);

    let mut placeholders = HashMap::new();

    placeholders.insert(
        "auth_player_name".to_string(),
        username.to_string(),
    );

    placeholders.insert(
        "auth_uuid".to_string(),
        uuid.to_string(),
    );

    placeholders.insert(
        "auth_access_token".to_string(),
        token.to_string(),
    );

    placeholders.insert(
        "user_type".to_string(),
        user_type.to_string(),
    );

    placeholders.insert(
        "version_name".to_string(),
        version_id.to_string(),
    );

    placeholders.insert(
        "version_type".to_string(),
        version_json
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("release")
            .to_string(),
    );

    placeholders.insert(
        "game_directory".to_string(),
        norm_game.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "assets_root".to_string(),
        norm_assets.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "assets_index_name".to_string(),
        asset_index_name,
    );

    placeholders.insert(
        "natives_directory".to_string(),
        norm_native.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "game_assets".to_string(),
        norm_assets.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "classpath".to_string(),
        final_classpath.clone(),
    );

    placeholders.insert(
        "library_directory".to_string(),
        loader_libs_path.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "classpath_separator".to_string(),
        classpath_separator.to_string(),
    );

    placeholders.insert("launcher_name".to_string(), crate::NAME.to_string());

    placeholders.insert("launcher_version".to_string(), crate::VERSION.to_string());

    placeholders.insert("clientid".to_string(), crate::CLIENT_ID.to_string());

    placeholders.insert("auth_xuid".to_string(), "placeholder_yet".to_string());

    let context = LaunchContext {
        my_os,
        my_arch,
        placeholders,
    };

    let mut jvm_args = Vec::new();

    jvm_args.push(format!("-Xmx{}", ram_count));

    jvm_args.push(format!(
        "-Djava.library.path={}",
        norm_native.display()
    ));

    if ely {
        if !betafix {
            let injector_path = norm_libs.join("authlib-injector.jar");

            if injector_path.exists() {
                jvm_args.push(format!(
                    "-javaagent:{}=ely.by",
                    injector_path.to_string_lossy()
                ));
            } else {
                println!(
                    "{}: Ely.by requested but authlib-injector.jar not found at {:?}",
                    musutils::types::Status::Warn.as_colored_str(),
                    injector_path
                );
            }
        } else {
            let proxy_port = 43523;

            jvm_args.push(
                "-Dhttp.proxyHost=127.0.0.1".to_string()
            );

            jvm_args.push(
                format!("-Dhttp.proxyPort={}", proxy_port)
            );

            jvm_args.push(
                "-Dhttp.nonProxyHosts=localhost|127.0.0.1".to_string()
            );
        }
    } else if betafix {
        jvm_args.push(
            "-Dhttp.proxyHost=betacraft.uk".to_string()
        );
    }

    let mut game_args = Vec::new();

    if let Some(arguments_obj) = version_json.get("arguments") {
        if let Some(jvm_value) = arguments_obj.get("jvm") {
            parse_argument_list(
                jvm_value,
                &context,
                &mut jvm_args,
            );
        }

        if let Some(game_value) = arguments_obj.get("game") {
            parse_argument_list(
                game_value,
                &context,
                &mut game_args,
            );
        }
    } else if let Some(minecraft_args_str) = version_json
        .get("minecraftArguments")
        .and_then(|a| a.as_str())
    {
        game_args.extend(
            minecraft_args_str
                .split_whitespace()
                .map(str::to_string),
        );
    }

    if let Some(ref loader_json) = modloader_json {
        if let Some(loader_args) = loader_json.get("arguments") {
            if let Some(jvm_value) = loader_args.get("jvm") {
                parse_argument_list(
                    jvm_value,
                    &context,
                    &mut jvm_args,
                );
            }

            if let Some(game_value) = loader_args.get("game") {
                parse_argument_list(
                    game_value,
                    &context,
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
            jvm_args.push(final_classpath.clone());
        }
    } else if !jvm_args.contains(&"-cp".to_string())
        && !jvm_args
            .iter()
            .any(|a| {
                a == "-classpath"
                    || a.starts_with("-classpath")
            })
    {
        jvm_args.push("-cp".to_string());
        jvm_args.push(final_classpath);
    }

    let final_jvm_args: Vec<String> = jvm_args
        .into_iter()
        .map(|arg| {
            replace_placeholders(
                &arg,
                &context.placeholders,
            )
        })
        .collect();

    let final_game_args: Vec<String> = game_args
        .into_iter()
        .map(|arg| {
            replace_placeholders(
                &arg,
                &context.placeholders,
            )
        })
        .collect();

    let mut command = Vec::new();

    command.push("java".to_string());
    command.extend(final_jvm_args);
    command.push(main_class);
    command.extend(final_game_args);

    Ok(command
        .into_iter()
        .map(|arg| {
            if arg.contains(' ') {
                format!("\"{}\"", arg)
            } else {
                arg
            }
        })
        .collect::<Vec<_>>()
        .join(" "))
}
