use serde_json::Value;

use crate::core::run::dto::LaunchContext;

fn normalize_arch(arch: &str) -> &'static str {
    match arch {
        "64" | "x86_64" | "amd64" | "x64" => "x86_64",
        "32" | "x86" | "i386" | "i686" => "x86",
        "arm64" | "aarch64" => "arm64",
        "arm" | "arm32" => "arm",
        _ => "unknown",
    }
}

fn rule_matches_os(filter: &Value, context: &LaunchContext) -> bool {
    if let Some(name) = filter.get("name").and_then(|n| n.as_str())
        && name != context.my_os
    {
        return false;
    }

    if let Some(arch) = filter.get("arch").and_then(|a| a.as_str())
        && normalize_arch(arch) != normalize_arch(&context.my_arch)
    {
        return false;
    }

    true
}

pub fn check_rules(rules: &[Value], context: &LaunchContext) -> bool {
    let mut allowed = false;
    for rule in rules {
        if rule.get("features").is_some() {
            continue;
        }

        let action = rule
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("allow");
        let is_allow = action == "allow";

        match rule.get("os") {
            Some(os_filter) => {
                if rule_matches_os(os_filter, context) {
                    allowed = is_allow;
                }
            }
            None => {
                allowed = is_allow;
            }
        }
    }
    allowed
}
