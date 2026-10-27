#![cfg(feature = "core")]

use std::collections::HashMap;

use rmlib::core::run::dto::LaunchContext;
use rmlib::core::run::{
    assemble_argv, assemble_command, check_rules, get_library_group_artifact,
    parse_argument_list, replace_placeholders, resolve_environment,
};
use serde_json::json;

fn ctx(os: &str, arch: &str) -> LaunchContext {
    LaunchContext {
        my_os: os.to_string(),
        my_arch: arch.to_string(),
        placeholders: HashMap::new(),
    }
}

fn ctx_with_placeholders(pairs: &[(&str, &str)]) -> LaunchContext {
    let mut map = HashMap::new();
    for (k, v) in pairs {
        map.insert((*k).to_string(), (*v).to_string());
    }
    LaunchContext {
        my_os: "linux".to_string(),
        my_arch: "64".to_string(),
        placeholders: map,
    }
}

#[test]
fn rules_empty_returns_false() {
    assert!(!check_rules(&[], &ctx("linux", "64")));
}

#[test]
fn rules_allow_without_os_matches() {
    let rules = vec![json!({"action": "allow"})];
    assert!(check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_disallow_without_os() {
    let rules = vec![json!({"action": "disallow"})];
    assert!(!check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_missing_action_defaults_to_allow() {
    let rules = vec![json!({})];
    assert!(check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_os_match() {
    let rules = vec![json!({"action": "allow", "os": {"name": "linux"}})];
    assert!(check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_os_mismatch_does_not_match() {
    let rules = vec![json!({"action": "allow", "os": {"name": "windows"}})];
    assert!(!check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_disallow_os_mismatch_leaves_default() {
    let rules = vec![json!({"action": "disallow", "os": {"name": "windows"}})];
    assert!(!check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_arch_normalized_match() {
    let rules = vec![json!({"action": "allow", "os": {"arch": "x86_64"}})];
    assert!(check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_arch_normalized_match_reverse() {
    let rules = vec![json!({"action": "allow", "os": {"arch": "64"}})];
    assert!(check_rules(&rules, &ctx("linux", "x86_64")));
}

#[test]
fn rules_arch_mismatch() {
    let rules = vec![json!({"action": "allow", "os": {"arch": "x86"}})];
    assert!(!check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_with_features_skipped() {
    let rules = vec![json!({"action": "allow", "features": {"is_demo_user": true}})];
    assert!(!check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn rules_last_matching_rule_wins() {
    let rules = vec![
        json!({"action": "allow"}),
        json!({"action": "disallow", "os": {"name": "linux"}}),
    ];
    assert!(!check_rules(&rules, &ctx("linux", "64")));
}

#[test]
fn library_group_artifact_basic() {
    assert_eq!(
        get_library_group_artifact("net.minecraft:client:1.20.1"),
        "net.minecraft:client"
    );
}

#[test]
fn library_group_artifact_two_parts() {
    assert_eq!(get_library_group_artifact("a:b"), "a:b");
}

#[test]
fn library_group_artifact_single_part() {
    assert_eq!(get_library_group_artifact("a"), "a");
}

#[test]
fn library_group_artifact_empty() {
    assert_eq!(get_library_group_artifact(""), "");
}

#[test]
fn replace_placeholders_basic() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Steve".to_string());
    assert_eq!(replace_placeholders("hi ${name}!", &map), "hi Steve!");
}

#[test]
fn replace_placeholders_multiple_occurrences() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), "1".to_string());
    assert_eq!(replace_placeholders("${x}+${x}", &map), "1+1");
}

#[test]
fn replace_placeholders_missing_key_leaves_text() {
    let map = HashMap::new();
    assert_eq!(replace_placeholders("${nope}", &map), "${nope}");
}

#[test]
fn replace_placeholders_no_placeholder() {
    let map = HashMap::new();
    assert_eq!(replace_placeholders("plain", &map), "plain");
}

#[test]
fn parse_argument_list_simple() {
    let value = json!(["-Xmx2G", "-Dfoo=bar"]);
    let mut out = Vec::new();
    parse_argument_list(&value, &ctx("linux", "64"), &mut out);
    assert_eq!(out, vec!["-Xmx2G", "-Dfoo=bar"]);
}

#[test]
fn parse_argument_list_conditional_single() {
    let value = json!([
        {"rules": [{"action": "allow", "os": {"name": "linux"}}], "value": "-Dlinux=1"}
    ]);
    let mut out = Vec::new();
    parse_argument_list(&value, &ctx("linux", "64"), &mut out);
    assert_eq!(out, vec!["-Dlinux=1"]);
}

#[test]
fn parse_argument_list_conditional_single_not_matching() {
    let value = json!([
        {"rules": [{"action": "allow", "os": {"name": "windows"}}], "value": "-Dwin=1"}
    ]);
    let mut out = Vec::new();
    parse_argument_list(&value, &ctx("linux", "64"), &mut out);
    assert!(out.is_empty());
}

#[test]
fn parse_argument_list_conditional_many() {
    let value = json!([
        {"rules": [{"action": "allow"}], "value": ["a", "b", "c"]}
    ]);
    let mut out = Vec::new();
    parse_argument_list(&value, &ctx("linux", "64"), &mut out);
    assert_eq!(out, vec!["a", "b", "c"]);
}

#[test]
fn parse_argument_list_not_array_is_noop() {
    let value = json!({"not": "array"});
    let mut out = Vec::new();
    parse_argument_list(&value, &ctx("linux", "64"), &mut out);
    assert!(out.is_empty());
}

#[test]
fn parse_argument_list_skips_invalid_elements() {
    let value = json!([42, null, ["nested"], "-Xmx1G"]);
    let mut out = Vec::new();
    parse_argument_list(&value, &ctx("linux", "64"), &mut out);
    assert_eq!(out, vec!["-Xmx1G"]);
}

#[test]
fn resolve_environment_matches_target_os() {
    let env = resolve_environment();
    if cfg!(target_os = "linux") {
        assert_eq!(env.my_os, "linux");
    } else if cfg!(target_os = "windows") {
        assert_eq!(env.my_os, "windows");
    } else if cfg!(target_os = "macos") {
        assert_eq!(env.my_os, "osx");
    }
}

#[test]
fn resolve_environment_matches_target_arch() {
    let env = resolve_environment();
    if cfg!(target_arch = "x86_64") {
        assert_eq!(env.my_arch, "64");
    } else if cfg!(target_arch = "x86") {
        assert_eq!(env.my_arch, "32");
    } else if cfg!(target_arch = "aarch64") {
        assert_eq!(env.my_arch, "arm64");
    } else if cfg!(target_arch = "arm") {
        assert_eq!(env.my_arch, "arm32");
    }
}

#[test]
fn assemble_argv_has_java_first_and_main_class() {
    let context = ctx_with_placeholders(&[("auth_player_name", "Steve")]);
    let args = rmlib::core::run::LaunchArguments {
        jvm_args: vec!["-Xmx2G".to_string(), "-Dfoo=${auth_player_name}".to_string()],
        game_args: vec!["--username".to_string(), "${auth_player_name}".to_string()],
    };
    let argv = assemble_argv("net.minecraft.client.main.Main", args, &context);
    assert_eq!(argv[0], "java");
    assert!(argv.contains(&"-Xmx2G".to_string()));
    assert!(argv.contains(&"-Dfoo=Steve".to_string()));
    assert!(argv.contains(&"net.minecraft.client.main.Main".to_string()));
    assert!(argv.contains(&"--username".to_string()));
    assert!(argv.contains(&"Steve".to_string()));
}

#[test]
fn assemble_argv_places_main_class_between_jvm_and_game_args() {
    let context = ctx("linux", "64");
    let args = rmlib::core::run::LaunchArguments {
        jvm_args: vec!["-Xmx1G".to_string()],
        game_args: vec!["--game".to_string()],
    };
    let argv = assemble_argv("Main", args, &context);
    assert_eq!(argv, vec!["java", "-Xmx1G", "Main", "--game"]);
}

#[test]
fn assemble_command_quotes_args_with_spaces() {
    let context = ctx("linux", "64");
    let args = rmlib::core::run::LaunchArguments {
        jvm_args: vec!["-Dkey=has space".to_string()],
        game_args: vec!["--x".to_string()],
    };
    let cmd = assemble_command("Main", args, &context);
    assert!(cmd.contains("\"-Dkey=has space\""));
    assert!(cmd.contains("Main"));
}

#[test]
fn assemble_command_no_quotes_without_spaces() {
    let context = ctx("linux", "64");
    let args = rmlib::core::run::LaunchArguments {
        jvm_args: vec!["-Xmx1G".to_string()],
        game_args: vec![],
    };
    let cmd = assemble_command("Main", args, &context);
    assert!(!cmd.contains('"'));
}
