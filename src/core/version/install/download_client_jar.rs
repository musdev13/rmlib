use std::path::Path;
use musutils;

pub async fn download_client_jar(
    version_json: &serde_json::Value,
    version_path: &Path,
    version_id: &str,
) {
    println!("{}: parsing and downloading client jar...", musutils::types::Status::Task.as_colored_str());
    
    let downloads = version_json.get("downloads")
        .expect(&format!("{}: 'downloads' field is missing", musutils::types::Status::Err.as_colored_str()));
    
    let client = downloads.get("client")
        .expect(&format!("{}: 'client' field is missing in downloads", musutils::types::Status::Err.as_colored_str()));
    
    let client_sha1 = client.get("sha1")
        .and_then(|v| v.as_str())
        .expect(&format!("{}: 'sha1' field is missing or not a string", musutils::types::Status::Err.as_colored_str()))
        .to_string();
    
    let client_url = client.get("url")
        .and_then(|v| v.as_str())
        .expect(&format!("{}: 'url' field is missing or not a string", musutils::types::Status::Err.as_colored_str()))
        .to_string();

    let jar_path = version_path.join(format!("{}.jar", version_id));

    musutils::http::download_to_sha1(client_url, jar_path, client_sha1).await
        .expect(&format!("{}: failed to download client jar", musutils::types::Status::Err.as_colored_str()));
}
