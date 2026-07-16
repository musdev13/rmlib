use std::path::PathBuf;

use crate::core;

pub async fn install(version_id: String, versions_path_option: Option<PathBuf>, libs_path_option: Option<PathBuf>, assets_path_option: Option<PathBuf>){
    let line = musutils::types::line::draw_colored('=', 35, musutils::color::Colors::Yellow);

    println!("{}: fetching `{}`...", musutils::types::Status::Task.as_colored_str(), &version_id);
    let version_json_str = core::version::get_version_json(version_id.clone()).await;
    println!("{}\n{}\n{}",line.clone(), version_json_str.clone(), line.clone());
    println!("{}: version `{}` found!", musutils::types::Status::Ok.as_colored_str(), version_id);

    println!("{}", line.clone());
    

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
    {
        let star = musutils::color::color_str("*", musutils::color::Colors::Orange); 
        println!("{}: got next paths:\n{}\n{}\n{}", musutils::types::Status::Ok.as_colored_str(),
            format!("{} {}", if is_custom_versions_path {star.clone()} else {"-".to_string()}, versions_path.clone().display() ),
            format!("{} {}", if is_custom_libs_path {star.clone()} else {"-".to_string()}, libs_path.clone().display() ),
            format!("{} {}", if is_custom_assets_path {star.clone()} else {"-".to_string()}, assets_path.clone().display() ),
        );
    }

}


