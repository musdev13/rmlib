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

    print!("{}: creating directories... ",musutils::types::Status::Task.as_colored_str());
    musutils::fs::new_dir(versions_path.clone());
    musutils::fs::new_dir(libs_path.clone());
    musutils::fs::new_dir(assets_path.clone());
    let version_path = versions_path.clone().join(version_id.clone());
    musutils::fs::new_dir(version_path.clone());
    println!("{}",musutils::types::Status::Ok.as_colored_str());
    
    println!("{}", line.clone());
    print!("{}: writing `{}.json`... ", musutils::types::Status::Task.as_colored_str(), version_id.clone());
    musutils::fs::write(version_path.clone().join(format!("{}{}", version_id.clone(), ".json")), version_json_str.clone()).expect(&format!("{}: can't write {}.json", musutils::types::Status::Err.as_colored_str(),version_id.clone()));
    println!("{}", musutils::types::Status::Ok.as_colored_str());
    
    

    println!("{}", line.clone());
    println!("{}: parsing and downloading client jar...", musutils::types::Status::Task.as_colored_str());
    
    let version_json: serde_json::Value = serde_json::from_str(&version_json_str)
    .expect(&format!("{}: can't parse client jar url", musutils::types::Status::Err.as_colored_str()));

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

    let jar_path = version_path.clone().join(format!("{}.jar", version_id.clone()));

    musutils::http::download_to_sha1(client_url, jar_path, client_sha1).await
        .expect(&format!("{}: failed to download client jar", musutils::types::Status::Err.as_colored_str()));
    
    println!("{}: we have client.jar! :3",musutils::types::Status::Ok.as_colored_str());

    

    println!("{}", line.clone());
    println!("{}: downloading libraries...", musutils::types::Status::Task.as_colored_str());
    
    let mut libs_downloader = musutils::http::AsyncDownloader::new(5, musutils::http::async_downloader::HashAlgo::Sha1);

    let my_os = match musutils::os::get_os() {
        musutils::os::OS::Windows => "windows",
        musutils::os::OS::Linux => "linux",
        musutils::os::OS::MacOs => "osx",
        musutils::os::OS::Unknown => "unknown",
    };

    libs_downloader.join().await.expect(&format!("{}: something happened with libs_downloader.. hehe... sowwy :3...",musutils::types::Status::Err.as_colored_str()));


}

