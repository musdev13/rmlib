use crate::core;

pub async fn get_version_json_url(version_id: String) -> String {
    let versions = core::version::fetch_list().await;

    let version_element = versions
        .iter()
        .find(|v| {
            v.get("id")
                .and_then(|id| id.as_str())
                .map(|id_str| id_str == version_id)
                .unwrap_or(false)
        })
        .expect(&format!("version with id '{}' not found", version_id));

    let url_value = musutils::fs::config::get_value(version_element, "url")
        .expect("url field is missing in the version element");

    let url: String = musutils::fs::config::parse_value_as(&url_value)
        .expect("url is not a valid string");

    url
}
