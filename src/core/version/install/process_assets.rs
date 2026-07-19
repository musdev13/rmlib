use std::fs;
use std::path::Path;
use musutils;

pub async fn process_assets(
    version_json: &serde_json::Value,
    assets_path: &Path,
    aassets: usize
) {
    println!("{}: processing assets...", musutils::types::Status::Task.as_colored_str());

    let asset_index_obj = version_json.get("assetIndex");
    
    let index_id = asset_index_obj
        .and_then(|i| i.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("legacy");

    let indexes_dir = assets_path.join("indexes");
    let objects_dir = assets_path.join("objects");
    let virtual_legacy_dir = assets_path.join("virtual").join("legacy");

    musutils::fs::new_dir(&indexes_dir);
    musutils::fs::new_dir(&objects_dir);

    let index_file_path = indexes_dir.join(format!("{}.json", index_id));

    if !musutils::fs::is_exist(&index_file_path) {
        if let Some(index_url) = asset_index_obj.and_then(|i| i.get("url")).and_then(|u| u.as_str()) {
            let mut index_downloader = musutils::http::AsyncDownloader::new(1, musutils::http::async_downloader::HashAlgo::Sha1);
            index_downloader.push(
                index_url.to_string(),
                index_file_path.clone(),
                asset_index_obj.and_then(|i| i.get("sha1")).and_then(|s| s.as_str()).map(|s| s.to_string()),
                None,
                None
            );
            index_downloader.join().await.ok();
        }
    }

    if let Ok(index_content) = fs::read_to_string(&index_file_path) {
        if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&index_content) {
            if let Some(objects) = index_json.get("objects").and_then(|o| o.as_object()) {
                let mut assets_downloader = musutils::http::AsyncDownloader::new(aassets, musutils::http::async_downloader::HashAlgo::Sha1);
                let mut legacy_copies = Vec::new();

                let is_legacy = version_json.get("assets")
                    .and_then(|a| a.as_str())
                    .map(|a| a == "legacy" || a == "pre-1.6")
                    .unwrap_or(index_id == "legacy");

                for (resource_name, desc) in objects {
                    if let Some(hash) = desc.get("hash").and_then(|h| h.as_str()) {
                        if hash.len() < 2 { continue; }
                        
                        let subdir = &hash[0..2];
                        let object_file_path = objects_dir.join(subdir).join(hash);
                        let final_url = format!("https://resources.download.minecraft.net/{}/{}", subdir, hash);

                        if is_legacy {
                            let legacy_target = virtual_legacy_dir.join(resource_name);
                            legacy_copies.push((object_file_path.clone(), legacy_target));
                        }

                        println!(
                            "{}: Adding asset to queue -> {} ({})",
                            musutils::types::Status::Inf.as_colored_str(),
                            resource_name,
                            hash
                        );

                        assets_downloader.push(
                            final_url,
                            object_file_path,
                            Some(hash.to_string()),
                            Some(format!("{}: {{1}} downloaded", musutils::types::Status::Ok.as_colored_str())),
                            Some(format!("{}: {{1}} is downloading now", musutils::types::Status::Inf.as_colored_str()))
                        );
                    }
                }

                println!("{}: downloading asset objects...", musutils::types::Status::Task.as_colored_str());
                assets_downloader.join().await.ok();
                println!("{}: asset objects downloaded", musutils::types::Status::Ok.as_colored_str());

                if is_legacy && !legacy_copies.is_empty() {
                    println!("{}: creating legacy virtual assets...", musutils::types::Status::Task.as_colored_str());
                    musutils::fs::new_dir(&virtual_legacy_dir);
                    
                    for (src, dst) in legacy_copies {
                        if musutils::fs::is_exist(&src) {
                            if let Some(parent) = dst.parent() {
                                musutils::fs::new_dir(parent);
                            }
                            fs::copy(&src, &dst).ok();
                        }
                    }
                    println!("{}: legacy virtual assets ready", musutils::types::Status::Ok.as_colored_str());
                }
            }
        }
    }

    println!("{}: assets processing finished successfully!", musutils::types::Status::Ok.as_colored_str());
}
