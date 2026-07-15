use serde_json::Value;

pub async fn fetch_list() -> Vec<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    
    let response_str = musutils::http::get_json_str(url.to_string()).await;

    let response: Value = serde_json::from_str(&response_str)
        .expect("Failed to parse manifest XML/JSON");

    response["versions"]
        .as_array()
        .expect("Failed to find 'versions' array in manifest")
        .clone()
}
