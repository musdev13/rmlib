use serde_json::Value;

pub async fn fetch_list() -> Vec<Value> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

    let response_str = musutils::http::get_json_str(url.to_string())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "{}: Failed to fetch manifest string",
                musutils::types::Status::Err.as_colored_str()
            )
        });

    let response: Value = serde_json::from_str(&response_str).unwrap_or_else(|_| {
        panic!(
            "{}: Failed to parse manifest XML/JSON",
            musutils::types::Status::Err.as_colored_str()
        )
    });

    response["versions"]
        .as_array()
        .unwrap_or_else(|| {
            panic!(
                "{}: Failed to find 'versions' array in manifest",
                musutils::types::Status::Err.as_colored_str()
            )
        })
        .clone()
}
