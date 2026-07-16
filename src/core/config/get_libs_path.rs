use crate::core::config::DEFAULT_PATHS_CONFIG;

pub fn get_libs_path() -> String {
    let config_str = musutils::fs::config::get(
        "rml", 
        "paths.json", 
        Some(&serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).unwrap())
    );
    let config_json = serde_json::from_str(&config_str).unwrap();

    let value_element = musutils::fs::config::get_value(&config_json, "libs_path")
        .expect("there is no libs_path in config");

    let libs_path: String = musutils::fs::config::parse_value_as(&value_element)
        .expect("libs_path is not a valid string");

    libs_path
}
