use musutils;
use std::fs;
use std::path::Path;

use musutils::http::HttpError;

fn print_errors(label: &str, errors: &[HttpError]) {
    eprintln!(
        "{}: {} failed: {} error(s)",
        musutils::types::Status::Err.as_colored_str(),
        label,
        errors.len()
    );
    for (i, e) in errors.iter().take(10).enumerate() {
        eprintln!("  {}. {}", i + 1, e);
    }
    if errors.len() > 10 {
        eprintln!("  ... and {} more", errors.len() - 10);
    }
}

pub async fn process_assets(
    version_json: &serde_json::Value,
    assets_path: &Path,
    aassets: usize,
    soft: bool,
) -> Result<(), Vec<HttpError>> {
    println!(
        "{}: processing assets...",
        musutils::types::Status::Task.as_colored_str()
    );

    let mut all_errors: Vec<HttpError> = Vec::new();

    let asset_index_obj = version_json.get("assetIndex");

    let index_id = asset_index_obj
        .and_then(|i| i.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("legacy");

    let indexes_dir = assets_path.join("indexes");
    let objects_dir = assets_path.join("objects");
    let virtual_legacy_dir = assets_path.join("virtual").join("legacy");

    if let Err(e) = musutils::fs::create_dir_all(&indexes_dir) {
        eprintln!(
            "{}: failed to create indexes dir: {}",
            musutils::types::Status::Err.as_colored_str(),
            e
        );
        return Err(all_errors);
    }
    if let Err(e) = musutils::fs::create_dir_all(&objects_dir) {
        eprintln!(
            "{}: failed to create objects dir: {}",
            musutils::types::Status::Err.as_colored_str(),
            e
        );
        return Err(all_errors);
    }

    let index_file_path = indexes_dir.join(format!("{}.json", index_id));

    if !musutils::fs::exists(&index_file_path)
        && let Some(index_url) = asset_index_obj
            .and_then(|i| i.get("url"))
            .and_then(|u| u.as_str())
    {
        let mut index_downloader = musutils::http::async_downloader::AsyncDownloader::new(
            1,
            musutils::http::async_downloader::HashAlgo::Sha1,
        );
        index_downloader.push(
            index_url.to_string(),
            index_file_path.clone(),
            asset_index_obj
                .and_then(|i| i.get("sha1"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string()),
            None,
            None,
        );
        if let Err(errors) = index_downloader.join().await {
            print_errors("asset index download", &errors);
            return Err(errors);
        }
    }

    let index_content = match fs::read_to_string(&index_file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "{}: failed to read asset index {}: {}",
                musutils::types::Status::Err.as_colored_str(),
                index_file_path.display(),
                e
            );
            return Err(all_errors);
        }
    };
    let index_json: serde_json::Value = match serde_json::from_str(&index_content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{}: failed to parse asset index: {}",
                musutils::types::Status::Err.as_colored_str(),
                e
            );
            return Err(all_errors);
        }
    };
    let objects = match index_json.get("objects").and_then(|o| o.as_object()) {
        Some(o) => o,
        None => {
            eprintln!(
                "{}: no `objects` in asset index",
                musutils::types::Status::Err.as_colored_str()
            );
            return Err(all_errors);
        }
    };

    let mut assets_downloader = musutils::http::async_downloader::AsyncDownloader::new(
        aassets,
        musutils::http::async_downloader::HashAlgo::Sha1,
    );
    let mut legacy_copies = Vec::new();

    let is_legacy = version_json
        .get("assets")
        .and_then(|a| a.as_str())
        .map(|a| a == "legacy" || a == "pre-1.6")
        .unwrap_or(index_id == "legacy");

    let mut queued = 0usize;

    for (resource_name, desc) in objects {
        if let Some(hash) = desc.get("hash").and_then(|h| h.as_str()) {
            if hash.len() < 2 {
                continue;
            }

            let subdir = &hash[0..2];
            let object_file_path = objects_dir.join(subdir).join(hash);
            let final_url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                subdir, hash
            );

            if is_legacy {
                let legacy_target = virtual_legacy_dir.join(resource_name);
                legacy_copies.push((object_file_path.clone(), legacy_target));
            }

            if soft && object_file_path.exists() {
                continue;
            }

            queued += 1;
            assets_downloader.push(
                final_url,
                object_file_path,
                Some(hash.to_string()),
                None,
                None,
            );
        }
    }

    println!(
        "{}: downloading {} asset object(s)...",
        musutils::types::Status::Task.as_colored_str(),
        queued
    );

    if let Err(errors) = assets_downloader.join().await {
        print_errors("asset objects download", &errors);
        all_errors.extend(errors);
    } else {
        println!(
            "{}: asset objects downloaded",
            musutils::types::Status::Ok.as_colored_str()
        );
    }

    if is_legacy && !legacy_copies.is_empty() {
        println!(
            "{}: creating legacy virtual assets...",
            musutils::types::Status::Task.as_colored_str()
        );
        if let Err(e) = musutils::fs::create_dir_all(&virtual_legacy_dir) {
            eprintln!(
                "{}: failed to create legacy virtual dir: {}",
                musutils::types::Status::Err.as_colored_str(),
                e
            );
            return Err(all_errors);
        }

        for (src, dst) in legacy_copies {
            if soft && dst.exists() {
                continue;
            }
            if musutils::fs::exists(&src) {
                if let Some(parent) = dst.parent() {
                    let _ = musutils::fs::create_dir_all(parent);
                }
                if let Err(e) = fs::copy(&src, &dst) {
                    eprintln!(
                        "{}: failed to copy legacy asset {} -> {}: {}",
                        musutils::types::Status::Warn.as_colored_str(),
                        src.display(),
                        dst.display(),
                        e
                    );
                }
            }
        }
        println!(
            "{}: legacy virtual assets ready",
            musutils::types::Status::Ok.as_colored_str()
        );
    }

    if all_errors.is_empty() {
        println!(
            "{}: assets processing finished successfully!",
            musutils::types::Status::Ok.as_colored_str()
        );
        Ok(())
    } else {
        eprintln!(
            "{}: assets processing finished with {} error(s)",
            musutils::types::Status::Err.as_colored_str(),
            all_errors.len()
        );
        Err(all_errors)
    }
}
