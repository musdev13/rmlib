use serde_json::Value;

use musutils::http::HttpError;
use std::path::PathBuf;

pub async fn install(
    version_id: String,
    fabric_version: String,
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
) {
    let line = musutils::types::line::draw_colored('=', 35, musutils::color::Colors::Yellow);

    println!(
        "{}: soft installing base version...",
        musutils::types::Status::Task.as_colored_str()
    );

    let soft = true;
    let iclient = true;
    let ilibs = true;
    let iassets = true;
    let alibs = 150;
    let aassets = 350;
    let json_path = None;

    crate::core::version::install::install(
        Some(version_id.clone()),
        json_path,
        versions_path_option.clone(),
        libs_path_option.clone(),
        assets_path_option.clone(),
        soft,
        iclient,
        ilibs,
        iassets,
        alibs,
        aassets,
    )
    .await;

    println!("{}", line.clone());
    println!(
        "{}: base version installed!",
        musutils::types::Status::Ok.as_colored_str()
    );
    println!("{}", line.clone());

    let (_, _, _, versions_path, libs_path, _) =
        crate::core::version::install::resolve_installation_paths(
            versions_path_option,
            libs_path_option,
            assets_path_option,
        );

    println!("{}", line.clone());

    let fabric_dir = versions_path
        .join("modloaders")
        .join("fabric")
        .join(&version_id);

    println!(
        "{}: creating Fabric version directories...",
        musutils::types::Status::Task.as_colored_str()
    );
    musutils::fs::new_dir(fabric_dir.clone());
    println!(
        "{}: version directories created!",
        musutils::types::Status::Ok.as_colored_str()
    );

    let fabric_libs_dir = libs_path.join("modloaders").join("fabric");

    println!(
        "{}: creating Fabric libs directories...",
        musutils::types::Status::Task.as_colored_str()
    );
    musutils::fs::new_dir(fabric_libs_dir.clone());
    println!(
        "{}: libs directories created!",
        musutils::types::Status::Ok.as_colored_str()
    );

    println!("{}", line.clone());

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        version_id, fabric_version
    );

    println!(
        "{}: fetching Fabric profile JSON for `{}`...",
        musutils::types::Status::Task.as_colored_str(),
        fabric_version
    );

    let json_str = musutils::http::get_json_str(url).await.unwrap_or_else(|_| {
        panic!(
            "{}: failed to fetch Fabric profile JSON",
            musutils::types::Status::Err.as_colored_str()
        )
    });

    println!(
        "{}: Fabric profile JSON fetched successfully!",
        musutils::types::Status::Ok.as_colored_str()
    );

    println!("{}", line.clone());

    let json_file_path = fabric_dir.join(format!("{}.json", fabric_version));

    println!(
        "{}: writing `{}.json`...",
        musutils::types::Status::Task.as_colored_str(),
        fabric_version
    );

    musutils::fs::write(json_file_path, json_str.clone()).unwrap_or_else(|_| {
        panic!(
            "{}: can't write {}.json",
            musutils::types::Status::Err.as_colored_str(),
            fabric_version
        )
    });

    println!(
        "{}: `{}.json` saved successfully!",
        musutils::types::Status::Ok.as_colored_str(),
        fabric_version
    );

    println!("{}", line.clone());

    let parsed_json: Value = serde_json::from_str(&json_str).unwrap_or_else(|_| {
        panic!(
            "{}: failed to parse Fabric profile JSON",
            musutils::types::Status::Err.as_colored_str()
        )
    });

    if let Err(errors) =
        download_fabric_libraries(&parsed_json, &fabric_libs_dir, &line, alibs, soft).await
    {
        eprintln!(
            "{}: Fabric libraries installation finished with {} error(s); rerun install to retry failed downloads",
            musutils::types::Status::Err.as_colored_str(),
            errors.len()
        );
    }
}

pub async fn download_fabric_libraries(
    version_json: &serde_json::Value,
    fabric_libs_path: &std::path::Path,
    line: &str,
    alibs: usize,
    soft: bool,
) -> Result<(), Vec<HttpError>> {
    println!(
        "{}: downloading Fabric libraries...",
        musutils::types::Status::Task.as_colored_str()
    );

    let mut libs_downloader = musutils::http::async_downloader::AsyncDownloader::new(
        alibs,
        musutils::http::async_downloader::HashAlgo::Sha1,
    );

    if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
        for library in libraries {
            let name = library.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }

            let maven_path = match crate::core::types::build_maven_path(name, None) {
                Some(p) => p,
                None => continue,
            };

            let base_url = library
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("https://libraries.minecraft.net/");
            let final_url = format!("{}{}", base_url, maven_path);
            let final_sha1 = library
                .get("sha1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let lib_path = fabric_libs_path.join(&maven_path);

            if soft && lib_path.exists() {
                continue;
            }

            libs_downloader.push(final_url, lib_path, final_sha1, None, None);
        }
    }

    println!(
        "{}: waiting downloading tasks",
        musutils::types::Status::Task.as_colored_str()
    );
    let join_result = libs_downloader.join().await;

    println!("{}", line);
    match join_result {
        Ok(()) => {
            println!(
                "{}: Fabric libraries downloaded",
                musutils::types::Status::Ok.as_colored_str()
            );
            Ok(())
        }
        Err(errors) => {
            eprintln!(
                "{}: Fabric libs download finished with {} error(s)",
                musutils::types::Status::Err.as_colored_str(),
                errors.len()
            );
            for (i, e) in errors.iter().take(10).enumerate() {
                eprintln!("  {}. {}", i + 1, e);
            }
            if errors.len() > 10 {
                eprintln!("  ... and {} more", errors.len() - 10);
            }
            Err(errors)
        }
    }
}
