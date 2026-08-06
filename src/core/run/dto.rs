use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonArgument {
    Simple(String),
    Conditional {
        rules: Vec<Value>,
        value: JsonArgValue,
    },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonArgValue {
    Single(String),
    Many(Vec<String>),
}

pub struct LaunchContext {
    pub my_os: String,
    pub my_arch: String,
    pub placeholders: HashMap<String, String>,
}

pub struct ResolvedLibrary {
    pub key: String,
    pub path: PathBuf,
}
