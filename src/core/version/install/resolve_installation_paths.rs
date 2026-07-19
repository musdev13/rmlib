use std::path::PathBuf;
use crate::core;
use musutils;

pub fn resolve_installation_paths(
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
) -> (bool, bool, bool, PathBuf, PathBuf, PathBuf) {
    println!("{}: resolving paths...", musutils::types::Status::Task.as_colored_str());

    let mut is_custom_versions_path = false;
    let mut is_custom_libs_path = false;
    let mut is_custom_assets_path = false;

    let versions_path = musutils::fs::tilda_desir(
        musutils::types::deoption(
            versions_path_option, 
            || PathBuf::from(core::config::get_versions_path()),
            &mut is_custom_versions_path
        )
    );

    let libs_path = musutils::fs::tilda_desir(
        musutils::types::deoption(
            libs_path_option, 
            || PathBuf::from(core::config::get_libs_path()),
            &mut is_custom_libs_path
        )
    );

    let assets_path = musutils::fs::tilda_desir(
        musutils::types::deoption(
            assets_path_option, 
            || PathBuf::from(core::config::get_assets_path()),
            &mut is_custom_assets_path
        )
    );

    let star = musutils::color::color_str("*", musutils::color::Colors::Orange); 
    println!(
        "{}: got next paths:\n{}\n{}\n{}", 
        musutils::types::Status::Ok.as_colored_str(),
        format!("{} {}", if is_custom_versions_path { star.clone() } else { "-".to_string() }, versions_path.display()),
        format!("{} {}", if is_custom_libs_path { star.clone() } else { "-".to_string() }, libs_path.display()),
        format!("{} {}", if is_custom_assets_path { star.clone() } else { "-".to_string() }, assets_path.display()),
    );

    (
        is_custom_versions_path,
        is_custom_libs_path,
        is_custom_assets_path,
        versions_path,
        libs_path,
        assets_path,
    )
}
