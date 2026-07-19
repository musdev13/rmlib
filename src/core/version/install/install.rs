use std::path::PathBuf;

use crate::core::version;

pub async fn install(
    version_id: Option<String>,
    json_path: Option<PathBuf>,
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
    soft: bool,
    iclient: bool,
    ilibs: bool,
    iassets: bool,
    alibs: usize,
    aassets: usize,
){
    let line = musutils::types::line::draw_colored('=', 35, musutils::color::Colors::Yellow);

    let (version_id, version_json_str) = version::install::resolve_version_data(version_id, json_path).await;

    println!("{}\n{}\n{}", line.clone(), version_json_str.clone(), line.clone());
    println!("{}: version `{}` found!", musutils::types::Status::Ok.as_colored_str(), version_id);

    println!("{}", line.clone());
    
    let (
        _is_custom_versions_path,
        _is_custom_libs_path,
        _is_custom_assets_path,
        versions_path,
        libs_path,
        assets_path,
    ) = version::install::resolve_installation_paths(versions_path_option, libs_path_option, assets_path_option);

    let version_path = version::install::prepare_version_files(
        &versions_path,
        &libs_path,
        &assets_path,
        &version_id,
        &version_json_str,
        &line,
    );

    let version_json: serde_json::Value = serde_json::from_str(&version_json_str)
        .expect(&format!("{}: can't parse client jar url", musutils::types::Status::Err.as_colored_str()));

    if iclient {
        println!("{}", line.clone());
        version::install::download_client_jar(&version_json, &version_path, &version_id, soft).await;
    }
    if ilibs {
        println!("{}", line.clone());
        version::install::download_libraries(&version_json, &libs_path, &line, alibs, soft).await;
        println!("{}", line.clone());
        version::install::extract_native_libraries(&libs_path);
    }
    if iassets{
        println!("{}", line.clone());
        version::install::process_assets(&version_json, &assets_path, aassets, soft).await;
    }
}
