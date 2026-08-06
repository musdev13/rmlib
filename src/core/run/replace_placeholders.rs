use std::collections::HashMap;

pub fn replace_placeholders(arg: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = arg.to_string();
    for (key, val) in placeholders {
        let target = format!("${{{}}}", key);
        result = result.replace(&target, val);
    }
    result
}
