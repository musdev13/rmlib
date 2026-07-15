use serde_json::{json, Value};
use std::sync::LazyLock;

pub static DEFAULT_PATHS_CONFIG: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "versions_path": "~/rml/versions",
        "assets_path": "~/rml/assets",
        "libs_path": "~/rml/libs"
    })
});

pub fn init(force: bool){
    println!("initializing...");
    let def_config_str = serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).unwrap();
    if force {
        println!("!!! FORCED !!!");
        musutils::fs::config::rewrite("rml", "paths.json", &def_config_str);
    } else {
        musutils::fs::config::get("rml", "paths.json", Some(&def_config_str));
    }
    println!("done :3\n\n!!! check your ~/.config or %AppData% directory for rml configuration !!!");
}
