use std::path::PathBuf;
use musutils;

pub fn prepare_version_files(
    versions_path: &PathBuf,
    libs_path: &PathBuf,
    assets_path: &PathBuf,
    version_id: &str,
    version_json_str: &str,
    line: &str,
) -> PathBuf {
    print!("{}: creating directories... ", musutils::types::Status::Task.as_colored_str());
    
    musutils::fs::new_dir(versions_path.clone());
    musutils::fs::new_dir(libs_path.clone());
    musutils::fs::new_dir(assets_path.clone());
    
    let version_path = versions_path.join(version_id);
    musutils::fs::new_dir(version_path.clone());
    println!("{}", musutils::types::Status::Ok.as_colored_str());
    
    println!("{}", line);
    
    print!("{}: writing `{}.json`... ", musutils::types::Status::Task.as_colored_str(), version_id);
    
    let json_file_path = version_path.join(format!("{}.json", version_id));
    musutils::fs::write(json_file_path, version_json_str.to_string()).expect(&format!(
        "{}: can't write {}.json", 
        musutils::types::Status::Err.as_colored_str(),
        version_id
    ));
    
    println!("{}", musutils::types::Status::Ok.as_colored_str());

    version_path
}
