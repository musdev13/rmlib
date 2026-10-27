use std::path::PathBuf;

pub async fn install_patch(
    libs_path_option: Option<PathBuf>,
    is_json: bool,
) -> std::io::Result<()> {
    let libs_path = musutils::fs::expand(
        libs_path_option.unwrap_or_else(|| PathBuf::from(crate::core::config::get_libs_path())),
    );

    let line = musutils::types::line::draw_colored('=', 35, musutils::color::Colors::Yellow);

    musutils::log!(is_json, "{line}");

    musutils::log!(
        is_json,
        "{} Installing Ely.by authlib patch",
        musutils::types::Status::Task.as_colored_str()
    );

    musutils::log!(is_json, "{line}");

    let json = musutils::http::get_github_json_str(
        "https://api.github.com/repos/yushijinhun/authlib-injector/releases".to_string(),
    )
    .await
    .map_err(|e| std::io::Error::other(e.to_string()))?;

    let releases: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let assets = releases
        .get(0)
        .and_then(|release| release.get("assets"))
        .and_then(|assets| assets.as_array())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "GitHub release assets not found",
            )
        })?;

    let download_url = assets
        .iter()
        .find(|asset| {
            asset.get("content_type").and_then(|value| value.as_str())
                == Some("application/java-archive")
        })
        .and_then(|asset| asset.get("browser_download_url"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "authlib-injector JAR not found in latest release",
            )
        })?;

    musutils::log!(
        is_json,
        "{} Found authlib-injector",
        musutils::types::Status::Ok.as_colored_str()
    );

    musutils::fs::new_dir(&libs_path);

    let path = libs_path.join("authlib-injector.jar");

    musutils::log!(
        is_json,
        "{} Downloading authlib-injector...",
        musutils::types::Status::Task.as_colored_str()
    );

    musutils::http::download_to(download_url.to_string(), path)
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    musutils::log!(
        is_json,
        "{} Authlib patch installed",
        musutils::types::Status::Ok.as_colored_str()
    );

    Ok(())
}
