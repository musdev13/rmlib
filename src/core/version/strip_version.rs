use serde_json::{Map, Value};

pub fn strip_version(
    versions: &[Value],
    keep_id: bool,
    keep_release_time: bool,
    keep_time: bool,
    keep_type: bool,
    keep_url: bool,
) -> Vec<Value> {
    versions
        .iter()
        .map(|version| {
            let mut stripped_object = Map::new();

            if let Some(obj) = version.as_object() {
                if keep_id {
                    if let Some(val) = obj.get("id") {
                        stripped_object.insert("id".to_string(), val.clone());
                    }
                }
                if keep_release_time {
                    if let Some(val) = obj.get("releaseTime") {
                        stripped_object.insert("releaseTime".to_string(), val.clone());
                    }
                }
                if keep_time {
                    if let Some(val) = obj.get("time") {
                        stripped_object.insert("time".to_string(), val.clone());
                    }
                }
                if keep_type {
                    if let Some(val) = obj.get("type") {
                        stripped_object.insert("type".to_string(), val.clone());
                    }
                }
                if keep_url {
                    if let Some(val) = obj.get("url") {
                        stripped_object.insert("url".to_string(), val.clone());
                    }
                }
            }

            Value::Object(stripped_object)
        })
        .collect()
}
