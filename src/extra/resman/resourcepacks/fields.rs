use std::path::Path;

use serde_json::{Map, Value};

use super::dto::ResourcepackInfo;
use super::error::RpError;
use super::source::Field;

pub fn info_to_value(info: &ResourcepackInfo) -> Value {
    serde_json::to_value(info).unwrap_or(Value::Null)
}

pub fn select_fields(info: &ResourcepackInfo, fields: &[Field]) -> Map<String, Value> {
    let full = info_to_value(info);
    let obj = match full.as_object() {
        Some(o) => o,
        None => return Map::new(),
    };
    let mut out = Map::new();
    for f in fields {
        let key = f.as_str();
        match obj.get(key) {
            Some(v) => out.insert(key.to_string(), v.clone()),
            None => out.insert(key.to_string(), Value::Null),
        };
    }
    out
}

pub fn info_file_fields(path: &Path, fields: &[Field]) -> Result<Map<String, Value>, RpError> {
    let info = super::info::info_from_file(path)?;
    Ok(select_fields(&info, fields))
}

pub fn scan_fields(dir: &Path, fields: &[Field]) -> Result<Vec<Map<String, Value>>, RpError> {
    let list = super::scan::scan_dir(dir)?;
    Ok(list.iter().map(|i| select_fields(i, fields)).collect())
}
