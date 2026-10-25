use std::collections::HashMap;

use crate::core::run::build_classpath::ClasspathData;
use crate::core::run::dto::LaunchContext;
use crate::core::run::load_version_data::VersionData;
use crate::core::run::prepare_paths::LaunchPaths;
use crate::core::run::resolve_environment::LaunchEnvironment;

#[allow(clippy::too_many_arguments)]
pub fn build_launch_context(
    version_id: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    version_data: &VersionData,
    paths: &LaunchPaths,
    environment: &LaunchEnvironment,
    classpath: &ClasspathData,
) -> LaunchContext {
    let user_type = if ely { "mojang" } else { "msa" };

    let mut placeholders = HashMap::new();

    placeholders.insert("auth_player_name".to_string(), username.to_string());

    placeholders.insert("auth_uuid".to_string(), uuid.to_string());

    placeholders.insert("auth_access_token".to_string(), token.to_string());

    placeholders.insert("user_type".to_string(), user_type.to_string());

    placeholders.insert("version_name".to_string(), version_id.to_string());

    placeholders.insert(
        "version_type".to_string(),
        version_data
            .version_json
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or("release")
            .to_string(),
    );

    placeholders.insert(
        "game_directory".to_string(),
        paths.game.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "assets_root".to_string(),
        paths.assets.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "assets_index_name".to_string(),
        version_data.asset_index_name.clone(),
    );

    placeholders.insert(
        "natives_directory".to_string(),
        paths.native.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "game_assets".to_string(),
        paths.assets.to_string_lossy().into_owned(),
    );

    placeholders.insert("classpath".to_string(), classpath.classpath.clone());

    placeholders.insert(
        "library_directory".to_string(),
        version_data.loader_libs_path.to_string_lossy().into_owned(),
    );

    placeholders.insert(
        "classpath_separator".to_string(),
        classpath.separator.clone(),
    );

    placeholders.insert("launcher_name".to_string(), crate::NAME.to_string());

    placeholders.insert("launcher_version".to_string(), crate::VERSION.to_string());

    placeholders.insert("clientid".to_string(), crate::CLIENT_ID.to_string());

    placeholders.insert("auth_xuid".to_string(), xuid.to_string());

    LaunchContext {
        my_os: environment.my_os.clone(),
        my_arch: environment.my_arch.clone(),
        placeholders,
    }
}
