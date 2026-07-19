use std::fs;
use std::io::Read;
use std::path::Path;
use musutils;
use sha1::{Digest, Sha1};

pub async fn download_client_jar(
    version_json: &serde_json::Value,
    version_path: &Path,
    version_id: &str,
    soft: bool,
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

    if soft && jar_path.exists() {
        if let Ok(mut file) = fs::File::open(&jar_path) {
            let mut hasher = Sha1::new();
            let mut buffer = [0; 8192];

            while let Ok(count) = file.read(&mut buffer) {
                if count == 0 { break; }
                hasher.update(&buffer[..count]);
            }

            let local_sha1 = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

            if local_sha1 == client_sha1 {
                println!("{}: client jar already exists and verified via sha1", musutils::types::Status::Ok.as_colored_str());
                return;
            }
        }
    }

    musutils::http::download_to_sha1(client_url, jar_path, client_sha1).await
        .expect(&format!("{}: failed to download client jar", musutils::types::Status::Err.as_colored_str()));
}
