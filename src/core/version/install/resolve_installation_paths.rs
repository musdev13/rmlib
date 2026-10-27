use std::path::PathBuf;

use crate::core;
use musutils;

pub fn resolve_installation_paths(
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
) -> (bool, bool, bool, PathBuf, PathBuf, PathBuf) {
    println!(
        "{}: resolving paths...",
        musutils::types::Status::Task.as_colored_str()
    );

    let is_custom_versions_path = versions_path_option.is_some();
    let is_custom_libs_path = libs_path_option.is_some();
    let is_custom_assets_path = assets_path_option.is_some();

    let versions_path = musutils::fs::expand(
        versions_path_option.unwrap_or_else(|| PathBuf::from(core::config::get_versions_path())),
    );
    let libs_path = musutils::fs::expand(
        libs_path_option.unwrap_or_else(|| PathBuf::from(core::config::get_libs_path())),
    );
    let assets_path = musutils::fs::expand(
        assets_path_option.unwrap_or_else(|| PathBuf::from(core::config::get_assets_path())),
    );

    let star = musutils::color::color_str("*", musutils::color::Colors::Orange);
    let mark_v = if is_custom_versions_path {
        star.clone()
    } else {
        "-".to_string()
    };
    let mark_l = if is_custom_libs_path {
        star.clone()
    } else {
        "-".to_string()
    };
    let mark_a = if is_custom_assets_path {
        star.clone()
    } else {
        "-".to_string()
    };
    println!(
        "{}: got next paths:\n{} {}\n{} {}\n{} {}",
        musutils::types::Status::Ok.as_colored_str(),
        mark_v,
        versions_path.display(),
        mark_l,
        libs_path.display(),
        mark_a,
        assets_path.display(),
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
