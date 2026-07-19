use std::path::PathBuf;
use std::fs;

use crate::core::version; 
use musutils;

pub async fn resolve_version_data(
    version_id: Option<String>,
    json_path: Option<PathBuf>,
) -> (String, String) {
    let mut is_id_found = false;
    let version_id_opt = musutils::types::deoption(version_id, || String::new(), &mut is_id_found);

    if is_id_found {
        println!(
            "{}: fetching `{}`...",
            musutils::types::Status::Task.as_colored_str(),
            &version_id_opt
        );
        let json_str = version::get_version_json(version_id_opt.clone()).await;
        (version_id_opt, json_str)
    } else if let Some(path) = json_path {
        let full_path = musutils::fs::tilda_desir(path);
        let json_str = fs::read_to_string(&full_path).expect(&format!(
            "{}: failed to read local json file",
            musutils::types::Status::Err.as_colored_str()
        ));

        let parsed_json: serde_json::Value = serde_json::from_str(&json_str).expect(&format!(
            "{}: invalid json format in local file",
            musutils::types::Status::Err.as_colored_str()
        ));

        let extracted_id = parsed_json
            .get("id")
            .and_then(|id| id.as_str())
            .expect(&format!(
                "{}: missing `id` field in local json",
                musutils::types::Status::Err.as_colored_str()
            ))
            .to_string();

        (extracted_id, json_str)
    } else {
        eprintln!(
            "{}: neither version_id nor json_path was provided",
            musutils::types::Status::Err.as_colored_str()
        );
        std::process::exit(1);
    }
}
