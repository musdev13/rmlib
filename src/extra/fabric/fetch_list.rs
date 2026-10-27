use serde_json::Value;

pub async fn fetch_list(version_id: &str) -> Vec<String> {
    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}",
        version_id
    );

    let response_str = musutils::http::get_json_str(url).await.unwrap_or_else(|_| {
        panic!(
            "{}: Failed to fetch Fabric manifest string",
            musutils::types::Status::Err.as_colored_str()
        )
    });

    let response: Value = serde_json::from_str(&response_str).unwrap_or_else(|_| {
        panic!(
            "{}: Failed to parse Fabric manifest JSON",
            musutils::types::Status::Err.as_colored_str()
        )
    });

    let items = response.as_array().unwrap_or_else(|| {
        panic!(
            "{}: Expected JSON array from Fabric API",
            musutils::types::Status::Err.as_colored_str()
        )
    });

    items
        .iter()
        .filter_map(|item| item["loader"]["version"].as_str().map(|s| s.to_string()))
        .collect()
}
