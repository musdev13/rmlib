use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use serde_json::Value;
use musutils;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum JsonArgument {
    Simple(String),
    Conditional {
        rules: Vec<Value>,
        value: JsonArgValue,
    },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum JsonArgValue {
    Single(String),
    Many(Vec<String>),
}

struct LaunchContext {
    my_os: String,
    my_arch: String,
    placeholders: HashMap<String, String>,
}

fn check_rules(rules: &[Value], context: &LaunchContext) -> bool {
    let mut allowed = false;
    for rule in rules {
        let action = rule.get("action").and_then(|a| a.as_str()).unwrap_or("allow");
        let is_allow = action == "allow";

        if let Some(os_filter) = rule.get("os") {
            if let Some(os_name) = os_filter.get("name").and_then(|n| n.as_str()) {
                if os_name == context.my_os {
                    if let Some(os_arch) = os_filter.get("arch").and_then(|a| a.as_str()) {
                        if os_arch == context.my_arch {
                            allowed = is_allow;
                        }
                    } else {
                        allowed = is_allow;
                    }
                }
            }
        } else if rule.get("features").is_some() {
            continue;
        } else {
            allowed = is_allow;
        }
    }
    allowed
}

fn parse_argument_list(args_value: &Value, context: &LaunchContext, out: &mut Vec<String>) {
    if let Some(arr) = args_value.as_array() {
        for item in arr {
            if let Ok(arg) = serde_json::from_value::<JsonArgument>(item.clone()) {
                match arg {
                    JsonArgument::Simple(s) => out.push(s),
                    JsonArgument::Conditional { rules, value } => {
                        if check_rules(&rules, context) {
                            match value {
                                JsonArgValue::Single(s) => out.push(s),
                                JsonArgValue::Many(v) => out.extend(v),
                            }
                        }
                    }
                }
            }
        }
    }
}

fn replace_placeholders(arg: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = arg.to_string();
    for (key, val) in placeholders {
        let target = format!("${{{}}}", key);
        result = result.replace(&target, val);
    }
    result
}

struct ResolvedLibrary {
    key: String,
    path: PathBuf,
}

/// Выделяет `group:artifact` из Maven-координат (например, `org.ow2.asm:asm:9.10.1` -> `org.ow2.asm:asm`)
fn get_library_group_artifact(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() >= 2 {
        format!("{}:{}", parts[0], parts[1])
    } else {
        name.to_string()
    }
}

fn verify_libraries(version_json: &Value, libs_path: &Path, context: &LaunchContext) -> Vec<ResolvedLibrary> {
    let mut classpath_entries = Vec::new();

    let os_native_key = match context.my_os.as_str() {
        "windows" => "natives-windows",
        "linux" => "natives-linux",
        "osx" => "natives-macos",
        _ => "unknown",
    };

    let build_maven_path = |name: &str| -> Option<String> {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 { return None; }
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        if parts.len() == 4 {
            Some(format!("{}/{}/{}/{}-{}-{}.jar", group, artifact, version, artifact, version, parts[3]))
        } else {
            Some(format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version))
        }
    };

    if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
        for library in libraries {
            let mut is_allowed = true;
            if let Some(rules) = library.get("rules").and_then(|r| r.as_array()) {
                is_allowed = check_rules(rules, context);
            }

            if !is_allowed { continue; }

            let name = library.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.is_empty() { continue; }

            // Фильтрация нативных наборов для чужих ОС
            if name.contains(":natives-") && !name.contains(os_native_key) {
                continue;
            }

            let mut rel_path = String::new();
            let native_classifier = library.get("natives")
                .and_then(|n| n.get(&context.my_os))
                .and_then(|c| c.as_str())
                .map(|c| c.replace("${arch}", &context.my_arch));

            if let Some(downloads) = library.get("downloads") {
                if let Some(ref classifier) = native_classifier {
                    if let Some(classifier_obj) = downloads.get("classifiers").and_then(|c| c.get(classifier)) {
                        rel_path = classifier_obj.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    }
                } else if let Some(artifact) = downloads.get("artifact") {
                    rel_path = artifact.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                }
            }

            if rel_path.is_empty() {
                if let Some(maven_path) = build_maven_path(name) {
                    rel_path = maven_path;
                }
            }

            if rel_path.is_empty() { continue; }

            // Ключ дедупликации без версии (group:artifact), плюс классификатор при наличии
            let group_artifact = get_library_group_artifact(name);
            let key = if let Some(ref classifier) = native_classifier {
                format!("{}:{}", group_artifact, classifier)
            } else {
                group_artifact
            };

            let full_lib_path = libs_path.join(&rel_path);

            // Проверяем существование файла. Если файл отсутствует, выводим предупреждение и НЕ добавляем в classpath
            if !full_lib_path.exists() {
                println!(
                    "{}: Missing library -> {}",
                    musutils::types::Status::Warn.as_colored_str(),
                    full_lib_path.display()
                );
            } else {
                classpath_entries.push(ResolvedLibrary { key, path: full_lib_path });
            }
        }
    }
    classpath_entries
}

pub async fn run_client(
    version_id: &str,
    fabric_version: Option<&str>,
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
) {
    println!("{}: preparing launch sequence...", musutils::types::Status::Task.as_colored_str());

    let mut _proxy_guard = None;

    let norm_versions = musutils::fs::tilda_desir(versions_path);
    let norm_assets = musutils::fs::tilda_desir(assets_path);
    let norm_libs = musutils::fs::tilda_desir(libs_path);
    let norm_native = musutils::fs::tilda_desir(native_path);
    let norm_game = musutils::fs::tilda_desir(game_path);

    if !norm_game.exists() {
        fs::create_dir_all(&norm_game).ok();
    }

    let json_path = norm_versions
        .join(version_id)
        .join(format!("{}.json", version_id));

    let client_jar_path = norm_versions
        .join(version_id)
        .join(format!("{}.jar", version_id));

    if !json_path.exists() {
        println!("{}: client json not found at {:?}", musutils::types::Status::Err.as_colored_str(), json_path);
        return;
    }

    let json_content = fs::read_to_string(&json_path)
        .expect(&format!("{}: failed to read client json", musutils::types::Status::Err.as_colored_str()));

    let version_json: Value = serde_json::from_str(&json_content)
        .expect(&format!("{}: failed to parse client json", musutils::types::Status::Err.as_colored_str()));

    let fabric_json: Option<Value> = if let Some(fab_ver) = fabric_version {
        let fabric_json_path = norm_versions
            .join("modloaders")
            .join("fabric")
            .join(version_id)
            .join(format!("{}.json", fab_ver));

        if fabric_json_path.exists() {
            println!("{}: loading Fabric profile JSON from {:?}", musutils::types::Status::Inf.as_colored_str(), fabric_json_path);
            let fab_content = fs::read_to_string(&fabric_json_path).ok();
            fab_content.and_then(|c| serde_json::from_str(&c).ok())
        } else {
            println!(
                "{}: Fabric version `{}` requested, but profile JSON was not found at {:?}",
                musutils::types::Status::Warn.as_colored_str(),
                fab_ver,
                fabric_json_path
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
    }.to_string();

    let my_arch = match musutils::os::get_arch() {
        musutils::os::Arch::X64 => "64",
        musutils::os::Arch::X86 => "32",
        musutils::os::Arch::Arm64 => "arm64",
        musutils::os::Arch::Arm => "arm32",
        _ => "unknown",
    }.to_string();

    let main_class = fabric_json
        .as_ref()
        .and_then(|fj| fj.get("mainClass"))
        .and_then(|m| m.as_str())
        .or_else(|| version_json.get("mainClass").and_then(|m| m.as_str()))
        .expect(&format!("{}: missing mainClass inside json", musutils::types::Status::Err.as_colored_str()))
        .to_string();

    let asset_index_name = version_json.get("assetIndex")
        .and_then(|a| a.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("legacy")
        .to_string();

    let user_type = if ely { "mojang" } else { "msa" };

    let mut raw_libs = Vec::new();

    // Приоритет отдаётся библиотекам Fabric
    // Поиск библиотек Fabric происходит в подпапке modloaders/fabric директории libs
    if let Some(ref fab_json) = fabric_json {
        let fabric_libs_path = norm_libs.join("modloaders").join("fabric");
        let fab_libs = verify_libraries(fab_json, &fabric_libs_path, &LaunchContext {
            my_os: my_os.clone(),
            my_arch: my_arch.clone(),
            placeholders: HashMap::new(),
        });
        raw_libs.extend(fab_libs);
    }

    let vanilla_libs = verify_libraries(&version_json, &norm_libs, &LaunchContext {
        my_os: my_os.clone(),
        my_arch: my_arch.clone(),
        placeholders: HashMap::new(),
    });
    raw_libs.extend(vanilla_libs);

    let mut seen_keys = std::collections::HashSet::new();
    let mut classpath_items = Vec::new();

    for lib in raw_libs {
        if seen_keys.insert(lib.key) {
            classpath_items.push(lib.path);
        }
    }

    if !client_jar_path.exists() {
        println!("{}: client jar is missing -> {:?}", musutils::types::Status::Warn.as_colored_str(), client_jar_path);
    } else {
        classpath_items.push(client_jar_path);
    }

    let classpath_strings: Vec<String> = classpath_items
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let classpath_separator = if cfg!(windows) { ";" } else { ":" };
    let final_classpath = classpath_strings.join(classpath_separator);

    let mut placeholders = HashMap::new();
    placeholders.insert("auth_player_name".to_string(), username.to_string());
    placeholders.insert("auth_uuid".to_string(), uuid.to_string());
    placeholders.insert("auth_access_token".to_string(), token.to_string());
    placeholders.insert("user_type".to_string(), user_type.to_string());
    placeholders.insert("version_name".to_string(), version_id.to_string());
    placeholders.insert("version_type".to_string(), version_json.get("type").and_then(|t| t.as_str()).unwrap_or("release").to_string());

    placeholders.insert("game_directory".to_string(), norm_game.to_string_lossy().into_owned());
    placeholders.insert("assets_root".to_string(), norm_assets.to_string_lossy().into_owned());
    placeholders.insert("assets_index_name".to_string(), asset_index_name);
    placeholders.insert("natives_directory".to_string(), norm_native.to_string_lossy().into_owned());
    placeholders.insert("game_assets".to_string(), norm_assets.to_string_lossy().into_owned());
    placeholders.insert("classpath".to_string(), final_classpath.clone());

    let context = LaunchContext {
        my_os,
        my_arch,
        placeholders,
    };

    let mut jvm_args = Vec::new();
    jvm_args.push(format!("-Xmx{}", ram_count));

    if ely {
        if !betafix {
            let injector_path = norm_libs.join("authlib-injector.jar");
            if injector_path.exists() {
                jvm_args.push(format!("-javaagent:{}=ely.by", injector_path.to_string_lossy()));
            } else {
                println!(
                    "{}: Ely.by requested but authlib-injector.jar not found at {:?}",
                    musutils::types::Status::Warn.as_colored_str(),
                    injector_path
                );
            }
        } else {
            let proxy_port = 43523;
            _proxy_guard = crate::extra::ely::start_beta_proxy(proxy_port);

            jvm_args.push("-Dhttp.proxyHost=127.0.0.1".to_string());
            jvm_args.push(format!("-Dhttp.proxyPort={}", proxy_port));
            jvm_args.push("-Dhttp.nonProxyHosts=localhost|127.0.0.1".to_string());
        }
    } else {
        if betafix {
            jvm_args.push("-Dhttp.proxyHost=betacraft.uk".to_string());
        }
    }

    let mut game_args = Vec::new();

    if let Some(arguments_obj) = version_json.get("arguments") {
        if let Some(jvm_value) = arguments_obj.get("jvm") {
            parse_argument_list(jvm_value, &context, &mut jvm_args);
        }

        if let Some(game_value) = arguments_obj.get("game") {
            parse_argument_list(game_value, &context, &mut game_args);
        }
    } else if let Some(minecraft_args_str) = version_json.get("minecraftArguments").and_then(|a| a.as_str()) {
        jvm_args.push(format!("-Djava.library.path={}", norm_native.display()));
        for split_arg in minecraft_args_str.split_whitespace() {
            game_args.push(split_arg.to_string());
        }
    }

    if let Some(ref fab_json) = fabric_json {
        if let Some(fab_args) = fab_json.get("arguments") {
            if let Some(jvm_value) = fab_args.get("jvm") {
                parse_argument_list(jvm_value, &context, &mut jvm_args);
            }
            if let Some(game_value) = fab_args.get("game") {
                parse_argument_list(game_value, &context, &mut game_args);
            }
        }
    }

    if !jvm_args.contains(&"-cp".to_string()) && !jvm_args.iter().any(|a| a.starts_with("-classpath")) {
        jvm_args.push("-cp".to_string());
        jvm_args.push(final_classpath);
    }

    let final_jvm_args: Vec<String> = jvm_args
        .into_iter()
        .map(|a| replace_placeholders(&a, &context.placeholders))
        .collect();

    let final_game_args: Vec<String> = game_args
        .into_iter()
        .map(|a| replace_placeholders(&a, &context.placeholders))
        .collect();

    println!("{}: spawning java process...", musutils::types::Status::Task.as_colored_str());

    let mut command = Command::new("java");
    command.args(&final_jvm_args);
    command.arg(&main_class);
    command.args(&final_game_args);
    command.current_dir(&norm_game);

    match command.spawn() {
        Ok(mut child) => {
            println!("{}: instance started successfully! Press Ctrl+C to exit.", musutils::types::Status::Ok.as_colored_str());

            match child.wait() {
                Ok(status) => println!("{}: game exited with status: {}", musutils::types::Status::Ok.as_colored_str(), status),
                Err(e) => println!("{}: error waiting for game process: {}", musutils::types::Status::Err.as_colored_str(), e),
            }
        }
        Err(e) => {
            println!("{}: failed to execute java process: {}", musutils::types::Status::Err.as_colored_str(), e);
        }
    }
}
