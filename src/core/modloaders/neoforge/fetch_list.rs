use roxmltree::Document;

pub async fn fetch_list(version_id: &str) -> Vec<String> {
    let prefix = match extract_neoforge_prefix(version_id) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml".to_string();

    let bytes = musutils::http::get_file(url)
        .await
        .expect(&format!("{}: Failed to fetch NeoForge maven-metadata string", musutils::types::Status::Err.as_colored_str()));

    let response_str = String::from_utf8(bytes)
        .expect(&format!("{}: Failed to parse NeoForge response as UTF-8", musutils::types::Status::Err.as_colored_str()));

    let doc = Document::parse(&response_str)
        .expect(&format!("{}: Failed to parse NeoForge XML", musutils::types::Status::Err.as_colored_str()));

    doc.descendants()
        .filter(|n| n.has_tag_name("version"))
        .filter_map(|n| n.text())
        .filter(|v| v.starts_with(&prefix))
        .map(|v| v.to_string())
        .rev()
        .collect()
}

fn extract_neoforge_prefix(version_id: &str) -> Option<String> {
    if let Some(stripped) = version_id.strip_prefix("1.") {
        if stripped.chars().all(|c| c.is_ascii_digit() || c == '.') {
            Some(format!("{}.", stripped))
        } else {
            None
        }
    } else if version_id.chars().all(|c| c.is_ascii_digit() || c == '.') {
        Some(format!("{}.", version_id))
    } else {
        None
    }
}
