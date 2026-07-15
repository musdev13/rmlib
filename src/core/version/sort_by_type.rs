use crate::core::types::VersionType;
use serde_json::Value;

pub fn sort_by_type(versions: &[Value], version_type: VersionType) -> Vec<Value> {
    let target_type = version_type.as_str();
    
    versions
        .iter()
        .filter(|v| v["type"].as_str() == Some(target_type))
        .cloned()
        .collect()
}
