use serde_json::Value;

use crate::core::run::check_rules;
use crate::core::run::dto::{JsonArgValue, JsonArgument, LaunchContext};

pub fn parse_argument_list(args_value: &Value, context: &LaunchContext, out: &mut Vec<String>) {
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
