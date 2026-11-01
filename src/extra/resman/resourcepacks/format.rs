use std::path::Path;

use serde_json::Value;

use super::error::RpError;
use super::fields::{info_file_fields, info_to_value, scan_fields, select_fields};
use super::source::Field;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Table,
    Plain,
}

impl Format {
    pub fn parse(s: &str) -> Result<Self, RpError> {
        match s {
            "json" => Ok(Format::Json),
            "table" => Ok(Format::Table),
            "plain" => Ok(Format::Plain),
            other => Err(RpError::InvalidArgument(format!(
                "unknown format {other:?} (expected json, table, plain)"
            ))),
        }
    }
}

fn supported_formats_as_string(v: &Value) -> Option<String> {
    match v {
        Value::Number(n) => Some(n.to_string()),
        Value::Object(map) => {
            let min = map.get("min_inclusive").and_then(|v| v.as_u64());
            let max = map.get("max_inclusive").and_then(|v| v.as_u64());
            match (min, max) {
                (Some(a), Some(b)) => Some(format!("{a}..{b}")),
                (Some(a), None) => Some(format!("{a}..")),
                (None, Some(b)) => Some(format!("..{b}")),
                _ => None,
            }
        }
        Value::Array(arr) => {
            let parts: Vec<String> = arr
                .iter()
                .map(|v| match v {
                    Value::Number(n) => n.to_string(),
                    other => musutils::vec::compact_json_value(other, 0),
                })
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(", "))
            }
        }
        Value::Null => None,
        other => Some(musutils::vec::compact_json_value(other, 0)),
    }
}

fn prepare_for_display(value: &Value) -> Value {
    fn walk(v: &Value) -> Value {
        match v {
            Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, val) in map {
                    if k == "supported_formats"
                        && let Some(repr) = supported_formats_as_string(val)
                    {
                        out.insert(k.clone(), Value::String(repr));
                        continue;
                    }
                    out.insert(k.clone(), walk(val));
                }
                Value::Object(out)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(walk).collect()),
            other => other.clone(),
        }
    }
    walk(value)
}

fn sanitize_strings(value: &Value) -> Value {
    match value {
        Value::String(s) => Value::String(
            s.chars()
                .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), sanitize_strings(v)))
                .collect(),
        ),
        Value::Array(arr) => Value::Array(arr.iter().map(sanitize_strings).collect()),
        other => other.clone(),
    }
}

pub fn render_value(value: &Value, format: Format) -> String {
    match format {
        Format::Json => serde_json::to_string_pretty(value).unwrap_or_default(),
        Format::Table => render_table(&sanitize_strings(&prepare_for_display(value))),
        Format::Plain => render_plain(&sanitize_strings(&prepare_for_display(value))),
    }
}

fn render_plain(value: &Value) -> String {
    match value {
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| format!("{k}: {}", musutils::vec::compact_json_value(v, 0)))
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Array(arr) => arr
            .iter()
            .map(render_plain)
            .collect::<Vec<_>>()
            .join("\n\n"),
        other => musutils::vec::compact_json_value(other, 0),
    }
}

fn render_table(value: &Value) -> String {
    let rows: Vec<Value> = match value {
        Value::Array(arr) => arr.clone(),
        Value::Object(_) => vec![value.clone()],
        other => return musutils::vec::compact_json_value(other, 0),
    };
    match musutils::vec::json_as_table(&rows) {
        Ok(s) => s,
        Err(musutils::vec::TableError::Empty) => "No data to display.".to_string(),
        Err(musutils::vec::TableError::NoFields) => "No fields to display.".to_string(),
    }
}

pub fn info_file_as(
    path: &Path,
    fields: Option<&[Field]>,
    format: Format,
) -> Result<String, RpError> {
    let info = super::info::info_from_file(path)?;
    let value = match fields {
        Some(f) => Value::Object(select_fields(&info, f)),
        None => info_to_value(&info),
    };
    Ok(render_value(&value, format))
}

pub fn scan_as(dir: &Path, fields: Option<&[Field]>, format: Format) -> Result<String, RpError> {
    let list = super::scan::scan_dir(dir)?;
    let value = match fields {
        Some(f) => Value::Array(
            list.iter()
                .map(|i| Value::Object(select_fields(i, f)))
                .collect(),
        ),
        None => Value::Array(list.iter().map(info_to_value).collect()),
    };
    Ok(render_value(&value, format))
}

pub fn info_fields_formatted(
    path: &Path,
    fields: &[Field],
    format: Format,
) -> Result<String, RpError> {
    let selected = info_file_fields(path, fields)?;
    Ok(render_value(&Value::Object(selected), format))
}

pub fn scan_fields_formatted(
    dir: &Path,
    fields: &[Field],
    format: Format,
) -> Result<String, RpError> {
    let selected = scan_fields(dir, fields)?;
    let value = Value::Array(selected.into_iter().map(Value::Object).collect());
    Ok(render_value(&value, format))
}
