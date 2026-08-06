use serde_json::Value;

use crate::core::run::dto::LaunchContext;

pub fn check_rules(rules: &[Value], context: &LaunchContext) -> bool {
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
