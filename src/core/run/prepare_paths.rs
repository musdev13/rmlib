use std::path::PathBuf;

pub struct LaunchPaths {
    pub versions: PathBuf,
    pub assets: PathBuf,
    pub libs: PathBuf,
    pub native: PathBuf,
    pub game: PathBuf,
    pub version_json: PathBuf,
    pub client_jar: PathBuf,
}

pub fn prepare_paths(
    version_id: &str,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
) -> Result<LaunchPaths, String> {
    let versions = musutils::fs::tilda_desir(versions_path);
    let assets = musutils::fs::tilda_desir(assets_path);
    let libs = musutils::fs::tilda_desir(libs_path);
    let native = musutils::fs::tilda_desir(native_path);
    let game = musutils::fs::tilda_desir(game_path);

    let version_json = versions
        .join(version_id)
        .join(format!("{}.json", version_id));

    let client_jar = versions
        .join(version_id)
        .join(format!("{}.jar", version_id));

    if !version_json.exists() {
        return Err(format!("client json not found at {:?}", version_json));
    }

    Ok(LaunchPaths {
        versions,
        assets,
        libs,
        native,
        game,
        version_json,
        client_jar,
    })
}
