#![cfg(feature = "core")]

mod common;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use common::TempDir;
use rmlib::core::run::dto::LaunchContext;
use rmlib::core::run::{
    LaunchEnvironment, VersionData, build_arguments, build_argv, build_classpath, build_command,
    build_launch_context, load_version_data, prepare_paths, verify_libraries,
};
use serde_json::{Value, json};

fn env() -> LaunchEnvironment {
    LaunchEnvironment {
        my_os: "linux".to_string(),
        my_arch: "64".to_string(),
    }
}

fn context() -> LaunchContext {
    LaunchContext {
        my_os: "linux".to_string(),
        my_arch: "64".to_string(),
        placeholders: HashMap::new(),
    }
}

fn minimal_version_json() -> Value {
    json!({
        "id": "1.20.1",
        "type": "release",
        "mainClass": "net.minecraft.client.main.Main",
        "assetIndex": { "id": "5" },
        "libraries": [],
        "arguments": {
            "jvm": ["-Djava.library.path=${natives_directory}"],
            "game": ["--username", "${auth_player_name}"]
        }
    })
}

#[test]
fn prepare_paths_success() {
    let dir = TempDir::new("prepare_ok");
    let versions = dir.join("versions");
    let libs = dir.join("libs");
    let assets = dir.join("assets");
    let native = dir.join("natives");
    let game = dir.join("game");

    let version_dir = versions.join("1.20.1");
    fs::create_dir_all(&version_dir).unwrap();
    fs::write(version_dir.join("1.20.1.json"), b"{}").unwrap();

    let paths = prepare_paths(
        "1.20.1",
        versions.to_str().unwrap(),
        assets.to_str().unwrap(),
        libs.to_str().unwrap(),
        native.to_str().unwrap(),
        game.to_str().unwrap(),
    )
    .unwrap();

    assert_eq!(paths.versions, musutils_expand(&versions));
    assert_eq!(
        paths.version_json,
        musutils_expand(&versions)
            .join("1.20.1")
            .join("1.20.1.json")
    );
    assert_eq!(
        paths.client_jar,
        musutils_expand(&versions).join("1.20.1").join("1.20.1.jar")
    );
}

#[test]
fn prepare_paths_missing_json_err() {
    let dir = TempDir::new("prepare_missing");
    let versions = dir.join("versions");
    fs::create_dir_all(&versions).unwrap();

    let err = prepare_paths(
        "1.20.1",
        versions.to_str().unwrap(),
        dir.join("assets").to_str().unwrap(),
        dir.join("libs").to_str().unwrap(),
        dir.join("native").to_str().unwrap(),
        dir.join("game").to_str().unwrap(),
    )
    .err()
    .unwrap();
    assert!(err.contains("client json not found"));
}

fn musutils_expand(p: &std::path::Path) -> PathBuf {
    let s = p.to_str().unwrap().to_string();
    let mut parts = Vec::new();
    for c in s.split('/') {
        match c {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let joined = parts.join("/");
    if s.starts_with('/') {
        PathBuf::from(format!("/{}", joined))
    } else {
        PathBuf::from(joined)
    }
}

#[test]
fn load_version_data_without_modloader() {
    let dir = TempDir::new("lvd_plain");
    let versions = dir.join("versions");
    let libs = dir.join("libs");
    let assets = dir.join("assets");
    let native = dir.join("natives");
    let game = dir.join("game");
    let version_dir = versions.join("1.20.1");
    fs::create_dir_all(&version_dir).unwrap();
    fs::write(
        version_dir.join("1.20.1.json"),
        minimal_version_json().to_string(),
    )
    .unwrap();

    let paths = prepare_paths(
        "1.20.1",
        versions.to_str().unwrap(),
        assets.to_str().unwrap(),
        libs.to_str().unwrap(),
        native.to_str().unwrap(),
        game.to_str().unwrap(),
    )
    .unwrap();

    let data = load_version_data("1.20.1", None, None, &paths, false).unwrap();
    assert_eq!(data.main_class, "net.minecraft.client.main.Main");
    assert_eq!(data.asset_index_name, "5");
    assert!(data.modloader_json.is_none());
    assert_eq!(data.loader_libs_path, musutils_expand(&libs));
}

#[test]
fn load_version_data_asset_index_legacy_fallback() {
    let dir = TempDir::new("lvd_legacy");
    let paths = make_paths_with_json(
        &dir,
        json!({
            "id": "1.20.1",
            "mainClass": "Main"
        }),
    );

    let data = load_version_data("1.20.1", None, None, &paths, false).unwrap();
    assert_eq!(data.asset_index_name, "legacy");
}

#[test]
fn load_version_data_missing_main_class_err() {
    let dir = TempDir::new("lvd_nomain");
    let paths = make_paths_with_json(&dir, json!({"id": "1.20.1"}));

    let err = load_version_data("1.20.1", None, None, &paths, false)
        .err()
        .unwrap();
    assert!(err.contains("mainClass"));
}

#[test]
fn load_version_data_fabric_profile_present() {
    let dir = TempDir::new("lvd_fabric");
    let paths = make_paths_with_json(&dir, minimal_version_json());

    let fabric_json = json!({"mainClass": "net.fabricmc.loader.impl.launch.knot.KnotClient"});
    let fabric_dir = paths
        .versions
        .join("modloaders")
        .join("fabric")
        .join("1.20.1");
    fs::create_dir_all(&fabric_dir).unwrap();
    fs::write(
        fabric_dir.join("0.15.0.json"),
        serde_json::to_vec(&fabric_json).unwrap(),
    )
    .unwrap();

    let data = load_version_data("1.20.1", Some("0.15.0"), None, &paths, false).unwrap();
    assert_eq!(
        data.main_class,
        "net.fabricmc.loader.impl.launch.knot.KnotClient"
    );
    assert!(data.modloader_json.is_some());
    assert_eq!(
        data.loader_libs_path,
        paths.libs.join("modloaders").join("fabric")
    );
}

#[test]
fn load_version_data_fabric_profile_missing() {
    let dir = TempDir::new("lvd_fabric_missing");
    let paths = make_paths_with_json(&dir, minimal_version_json());

    let data = load_version_data("1.20.1", Some("0.99.0"), None, &paths, false).unwrap();
    assert_eq!(data.main_class, "net.minecraft.client.main.Main");
    assert!(data.modloader_json.is_none());
}

#[test]
fn load_version_data_neoforge_profile_present() {
    let dir = TempDir::new("lvd_neo");
    let paths = make_paths_with_json(&dir, minimal_version_json());

    let neoforge_json = json!({"mainClass": "cpw.mods.bootstraplauncher.BootstrapLauncher"});
    let neo_dir = paths
        .versions
        .join("modloaders")
        .join("neoforge")
        .join("1.20.1")
        .join("20.4.100");
    fs::create_dir_all(&neo_dir).unwrap();
    fs::write(
        neo_dir.join("20.4.100.json"),
        serde_json::to_vec(&neoforge_json).unwrap(),
    )
    .unwrap();

    let data = load_version_data("1.20.1", None, Some("20.4.100"), &paths, false).unwrap();
    assert_eq!(
        data.main_class,
        "cpw.mods.bootstraplauncher.BootstrapLauncher"
    );
    assert_eq!(
        data.loader_libs_path,
        paths.libs.join("modloaders").join("neoforge")
    );
}

#[test]
fn load_version_data_missing_version_json_err() {
    let dir = TempDir::new("lvd_missing");
    let paths = make_paths_with_json(&dir, minimal_version_json());
    fs::remove_file(&paths.version_json).unwrap();

    let err = load_version_data("1.20.1", None, None, &paths, false)
        .err()
        .unwrap();
    assert!(err.contains("failed to read client json"));
}

fn make_paths_with_json(dir: &TempDir, json_value: Value) -> rmlib::core::run::LaunchPaths {
    let versions = dir.join("versions");
    let libs = dir.join("libs");
    let assets = dir.join("assets");
    let native = dir.join("natives");
    let game = dir.join("game");
    let version_dir = versions.join("1.20.1");
    fs::create_dir_all(&version_dir).unwrap();
    fs::write(
        version_dir.join("1.20.1.json"),
        serde_json::to_vec(&json_value).unwrap(),
    )
    .unwrap();
    prepare_paths(
        "1.20.1",
        versions.to_str().unwrap(),
        assets.to_str().unwrap(),
        libs.to_str().unwrap(),
        native.to_str().unwrap(),
        game.to_str().unwrap(),
    )
    .unwrap()
}

#[test]
fn verify_libraries_only_existing_files_are_returned() {
    let dir = TempDir::new("vl_exist");
    let libs = dir.join("libs");
    fs::create_dir_all(&libs).unwrap();
    let jar_rel = "com/example/lib/1.0/lib-1.0.jar";
    let jar_path = libs.join(jar_rel);
    fs::create_dir_all(jar_path.parent().unwrap()).unwrap();
    fs::write(&jar_path, b"jar").unwrap();

    let version_json = json!({
        "libraries": [
            {"name": "com.example:lib:1.0", "downloads": {"artifact": {"path": jar_rel}}},
            {"name": "com.example:missing:1.0", "downloads": {"artifact": {"path": "com/example/missing/1.0/missing-1.0.jar"}}}
        ]
    });

    let out = verify_libraries(&version_json, &libs, &context());
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].key, "com.example:lib");
    assert_eq!(out[0].path, jar_path);
}

#[test]
fn verify_libraries_skips_by_rules() {
    let dir = TempDir::new("vl_rules");
    let libs = dir.join("libs");
    let jar = libs.join("a.jar");
    fs::create_dir_all(&libs).unwrap();
    fs::write(&jar, b"jar").unwrap();

    let version_json = json!({
        "libraries": [
            {
                "name": "x:y:1",
                "rules": [{"action": "allow", "os": {"name": "windows"}}],
                "downloads": {"artifact": {"path": "a.jar"}}
            }
        ]
    });

    let out = verify_libraries(&version_json, &libs, &context());
    assert!(out.is_empty());
}

#[test]
fn verify_libraries_no_libraries_field() {
    let dir = TempDir::new("vl_empty");
    let libs = dir.join("libs");
    fs::create_dir_all(&libs).unwrap();
    let out = verify_libraries(&json!({}), &libs, &context());
    assert!(out.is_empty());
}

#[test]
fn build_classpath_deduplicates_and_adds_client_jar() {
    let dir = TempDir::new("cp_dedup");
    let libs = dir.join("libs");
    let versions = dir.join("versions");
    fs::create_dir_all(&libs).unwrap();
    let jar_rel = "com/x/lib/1.0/lib-1.0.jar";
    let jar = libs.join(jar_rel);
    fs::create_dir_all(jar.parent().unwrap()).unwrap();
    fs::write(&jar, b"jar").unwrap();

    let client_jar = versions.join("1.20.1").join("1.20.1.jar");
    fs::create_dir_all(client_jar.parent().unwrap()).unwrap();
    fs::write(&client_jar, b"client").unwrap();

    let paths = rmlib::core::run::LaunchPaths {
        versions: versions.clone(),
        assets: dir.join("assets"),
        libs: libs.clone(),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: versions.join("1.20.1").join("1.20.1.json"),
        client_jar,
    };

    let version_json = json!({
        "libraries": [
            {"name": "com.x:lib:1.0", "downloads": {"artifact": {"path": jar_rel}}},
            {"name": "com.x:lib:1.0", "downloads": {"artifact": {"path": jar_rel}}}
        ]
    });

    let cp = build_classpath(&version_json, None, &paths, &libs, &env());
    let sep = if cfg!(windows) { ";" } else { ":" };
    assert_eq!(cp.separator, sep);

    let entries: Vec<&str> = cp.classpath.split(sep).collect();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|e| e.ends_with("lib-1.0.jar")));
    assert!(entries.iter().any(|e| e.ends_with("1.20.1.jar")));
}

#[test]
fn build_launch_context_placeholders() {
    let dir = TempDir::new("ctx_placeholders");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };

    let data = VersionData {
        version_json: json!({"type": "release"}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };

    let cp = rmlib::core::run::ClasspathData {
        classpath: "a:b".to_string(),
        separator: ":".to_string(),
    };

    let context = build_launch_context(
        "1.20.1",
        "Steve",
        "uuid-123",
        "xuid-1",
        "token-abc",
        false,
        &data,
        &paths,
        &env(),
        &cp,
    );

    assert_eq!(context.my_os, "linux");
    assert_eq!(context.my_arch, "64");
    assert_eq!(
        context.placeholders.get("auth_player_name").unwrap(),
        "Steve"
    );
    assert_eq!(context.placeholders.get("auth_uuid").unwrap(), "uuid-123");
    assert_eq!(
        context.placeholders.get("auth_access_token").unwrap(),
        "token-abc"
    );
    assert_eq!(context.placeholders.get("auth_xuid").unwrap(), "xuid-1");
    assert_eq!(context.placeholders.get("user_type").unwrap(), "msa");
    assert_eq!(context.placeholders.get("version_name").unwrap(), "1.20.1");
    assert_eq!(context.placeholders.get("version_type").unwrap(), "release");
    assert_eq!(context.placeholders.get("assets_index_name").unwrap(), "5");
    assert_eq!(context.placeholders.get("classpath").unwrap(), "a:b");
    assert_eq!(
        context.placeholders.get("classpath_separator").unwrap(),
        ":"
    );
    assert_eq!(
        context.placeholders.get("launcher_name").unwrap(),
        rmlib::NAME
    );
    assert_eq!(
        context.placeholders.get("launcher_version").unwrap(),
        rmlib::VERSION
    );
    assert_eq!(
        context.placeholders.get("clientid").unwrap(),
        rmlib::CLIENT_ID
    );
}

#[test]
fn build_launch_context_user_type_ely_is_mojang() {
    let dir = TempDir::new("ctx_ely");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({"type": "release"}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "x".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "Steve",
        "u",
        "x",
        "t",
        true,
        &data,
        &paths,
        &env(),
        &cp,
    );
    assert_eq!(context.placeholders.get("user_type").unwrap(), "mojang");
}

#[test]
fn build_arguments_includes_ram_and_library_path() {
    let dir = TempDir::new("args_basic");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({"arguments": {"jvm": ["-Dfoo=bar"], "game": ["--x"]}}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "cp.jar".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "Steve",
        "u",
        "x",
        "t",
        false,
        &data,
        &paths,
        &env(),
        &cp,
    );

    let args = build_arguments(
        "2G",
        false,
        false,
        None,
        &paths,
        &data,
        &context,
        &cp,
        &[],
        &[],
        false,
    );

    assert!(args.jvm_args.contains(&"-Xmx2G".to_string()));
    assert!(
        args.jvm_args
            .iter()
            .any(|a| a.starts_with("-Djava.library.path="))
    );
    assert!(args.jvm_args.contains(&"-Dfoo=bar".to_string()));
    assert!(args.jvm_args.contains(&"-cp".to_string()));
    assert!(args.jvm_args.contains(&"cp.jar".to_string()));
    assert!(args.game_args.contains(&"--x".to_string()));
}

#[test]
fn build_arguments_custom_jvm_and_game_args_appended() {
    let dir = TempDir::new("args_custom");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({"arguments": {"jvm": [], "game": []}}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "cp.jar".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "S",
        "u",
        "x",
        "t",
        false,
        &data,
        &paths,
        &env(),
        &cp,
    );

    let args = build_arguments(
        "1G",
        false,
        false,
        None,
        &paths,
        &data,
        &context,
        &cp,
        &["-Dcustom=1".to_string()],
        &["--custom".to_string()],
        false,
    );
    assert!(args.jvm_args.contains(&"-Dcustom=1".to_string()));
    assert!(args.game_args.contains(&"--custom".to_string()));
}

#[test]
fn build_arguments_minecraft_arguments_field_used_for_old_versions() {
    let dir = TempDir::new("args_legacy");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({"minecraftArguments": "--username ${auth_player_name} --gameDir ${game_directory}"}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "cp.jar".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "Steve",
        "u",
        "x",
        "t",
        false,
        &data,
        &paths,
        &env(),
        &cp,
    );

    let args = build_arguments(
        "2G",
        false,
        false,
        None,
        &paths,
        &data,
        &context,
        &cp,
        &[],
        &[],
        false,
    );
    assert!(args.game_args.contains(&"--username".to_string()));
    assert!(args.game_args.contains(&"${auth_player_name}".to_string()));
    assert!(args.game_args.contains(&"--gameDir".to_string()));
}

#[test]
fn build_arguments_ely_adds_javaagent_when_jar_exists() {
    let dir = TempDir::new("args_ely");
    let libs = dir.join("libs");
    let injector = libs.join("authlib-injector.jar");
    fs::create_dir_all(&libs).unwrap();
    fs::write(&injector, b"jar").unwrap();

    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: libs.clone(),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "cp.jar".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "S",
        "u",
        "x",
        "t",
        true,
        &data,
        &paths,
        &env(),
        &cp,
    );

    let args = build_arguments(
        "2G",
        true,
        false,
        None,
        &paths,
        &data,
        &context,
        &cp,
        &[],
        &[],
        false,
    );
    assert!(
        args.jvm_args
            .iter()
            .any(|a| a.starts_with("-javaagent:") && a.contains("authlib-injector.jar"))
    );
}

#[test]
fn build_arguments_ely_betafix_uses_proxy_port() {
    let dir = TempDir::new("args_betafix");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "cp.jar".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "S",
        "u",
        "x",
        "t",
        true,
        &data,
        &paths,
        &env(),
        &cp,
    );

    let args = build_arguments(
        "2G",
        true,
        true,
        None,
        &paths,
        &data,
        &context,
        &cp,
        &[],
        &[],
        false,
    );
    assert!(
        args.jvm_args
            .iter()
            .any(|a| a == "-Dhttp.proxyHost=127.0.0.1")
    );
    assert!(args.jvm_args.iter().any(|a| a == "-Dhttp.proxyPort=43523"));
}

#[test]
fn build_arguments_betafix_without_ely() {
    let dir = TempDir::new("args_bf_only");
    let paths = rmlib::core::run::LaunchPaths {
        versions: dir.join("versions"),
        assets: dir.join("assets"),
        libs: dir.join("libs"),
        native: dir.join("natives"),
        game: dir.join("game"),
        version_json: PathBuf::new(),
        client_jar: PathBuf::new(),
    };
    let data = VersionData {
        version_json: json!({}),
        modloader_json: None,
        main_class: "Main".to_string(),
        asset_index_name: "5".to_string(),
        loader_libs_path: paths.libs.clone(),
    };
    let cp = rmlib::core::run::ClasspathData {
        classpath: "cp.jar".to_string(),
        separator: ":".to_string(),
    };
    let context = build_launch_context(
        "1.20.1",
        "S",
        "u",
        "x",
        "t",
        false,
        &data,
        &paths,
        &env(),
        &cp,
    );

    let args = build_arguments(
        "2G",
        false,
        true,
        None,
        &paths,
        &data,
        &context,
        &cp,
        &[],
        &[],
        false,
    );
    assert!(
        args.jvm_args
            .iter()
            .any(|a| a == "-Dhttp.proxyHost=betacraft.uk")
    );
}

#[test]
fn build_argv_end_to_end() {
    let dir = TempDir::new("build_argv");
    let paths = make_paths_with_json(&dir, minimal_version_json());

    let argv = build_argv(
        "1.20.1",
        None,
        None,
        "1G",
        "Steve",
        "uuid-1",
        "xuid-1",
        "token",
        false,
        false,
        paths.versions.to_str().unwrap(),
        paths.assets.to_str().unwrap(),
        paths.libs.to_str().unwrap(),
        paths.native.to_str().unwrap(),
        paths.game.to_str().unwrap(),
        &[],
        &[],
        false,
    )
    .unwrap();

    assert_eq!(argv[0], "java");
    assert!(argv.contains(&"net.minecraft.client.main.Main".to_string()));
    assert!(argv.contains(&"-Xmx1G".to_string()));
    assert!(argv.contains(&"Steve".to_string()));
    assert!(argv.contains(&"--username".to_string()));
}

#[test]
fn build_command_returns_string() {
    let dir = TempDir::new("build_command");
    let paths = make_paths_with_json(&dir, minimal_version_json());

    let cmd = build_command(
        "1.20.1",
        None,
        None,
        "1G",
        "Steve",
        "u",
        "x",
        "t",
        false,
        false,
        paths.versions.to_str().unwrap(),
        paths.assets.to_str().unwrap(),
        paths.libs.to_str().unwrap(),
        paths.native.to_str().unwrap(),
        paths.game.to_str().unwrap(),
        &[],
        &[],
        false,
    )
    .unwrap();

    assert!(cmd.starts_with("java "));
    assert!(cmd.contains("net.minecraft.client.main.Main"));
    assert!(cmd.contains("-Xmx1G"));
}

#[test]
fn build_argv_missing_version_json_err() {
    let dir = TempDir::new("build_argv_missing");
    let paths = make_paths_with_json(&dir, minimal_version_json());
    fs::remove_file(&paths.version_json).unwrap();

    let err = build_argv(
        "1.20.1",
        None,
        None,
        "1G",
        "S",
        "u",
        "x",
        "t",
        false,
        false,
        paths.versions.to_str().unwrap(),
        paths.assets.to_str().unwrap(),
        paths.libs.to_str().unwrap(),
        paths.native.to_str().unwrap(),
        paths.game.to_str().unwrap(),
        &[],
        &[],
        false,
    )
    .unwrap_err();
    assert!(err.contains("client json not found"));
}
