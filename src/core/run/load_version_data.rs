use std::fs;

use serde_json::Value;

use crate::core::run::prepare_paths::LaunchPaths;

pub struct VersionData {
    pub version_json: Value,
    pub modloader_json: Option<Value>,
    pub main_class: String,
    pub asset_index_name: String,
    pub loader_libs_path: std::path::PathBuf,
}

pub fn load_version_data(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    paths: &LaunchPaths,
) -> Result<VersionData, String> {
    let json_content = fs::read_to_string(&paths.version_json)
        .map_err(|e| format!("failed to read client json: {}", e))?;

    let version_json: Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("failed to parse client json: {}", e))?;

    let modloader_json = if let Some(fab_ver) = fabric_version {
        let fabric_json_path = paths
            .versions
            .join("modloaders")
            .join("fabric")
            .join(version_id)
            .join(format!("{}.json", fab_ver));

        if fabric_json_path.exists() {
            println!(
                "{}: loading Fabric profile JSON from {:?}",
                musutils::types::Status::Inf.as_colored_str(),
                fabric_json_path
            );

            fs::read_to_string(&fabric_json_path)
                .ok()
                .and_then(|content| {
                    serde_json::from_str::<Value>(&content).ok()
                })
        } else {
            println!(
                "{}: Fabric version `{}` requested, but profile JSON was not found at {:?}",
                musutils::types::Status::Warn.as_colored_str(),
                fab_ver,
                fabric_json_path
            );

            None
        }
    } else if let Some(neoforge_ver) = neoforge_version {
        let neoforge_json_path = paths
            .versions
            .join("modloaders")
            .join("neoforge")
            .join(version_id)
            .join(neoforge_ver)
            .join(format!("{}.json", neoforge_ver));

        if neoforge_json_path.exists() {
            println!(
                "{}: loading NeoForge profile JSON from {:?}",
                musutils::types::Status::Inf.as_colored_str(),
                neoforge_json_path
            );

            fs::read_to_string(&neoforge_json_path)
                .ok()
                .and_then(|content| {
                    serde_json::from_str::<Value>(&content).ok()
                })
        } else {
            println!(
                "{}: NeoForge version `{}` requested, but profile JSON was not found at {:?}",
                musutils::types::Status::Warn.as_colored_str(),
                neoforge_ver,
                neoforge_json_path
            );

            None
        }
    } else {
        None
    };

    let main_class = modloader_json
        .as_ref()
        .and_then(|json| json.get("mainClass"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            version_json
                .get("mainClass")
                .and_then(|value| value.as_str())
        })
        .ok_or_else(|| "missing mainClass inside json".to_string())?
        .to_string();

    let asset_index_name = version_json
        .get("assetIndex")
        .and_then(|asset_index| asset_index.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("legacy")
        .to_string();

    let loader_libs_path = if fabric_version.is_some() {
        paths.libs.join("modloaders").join("fabric")
    } else if neoforge_version.is_some() {
        paths.libs.join("modloaders").join("neoforge")
    } else {
        paths.libs.clone()
    };

    Ok(VersionData {
        version_json,
        modloader_json,
        main_class,
        asset_index_name,
        loader_libs_path,
    })
}
