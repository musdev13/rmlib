use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::Value;

use crate::core::run::dto::LaunchContext;
use crate::core::run::prepare_paths::LaunchPaths;
use crate::core::run::resolve_environment::LaunchEnvironment;
use crate::core::run::verify_libraries::verify_libraries;

pub struct ClasspathData {
    pub classpath: String,
    pub separator: String,
}

pub fn build_classpath(
    version_json: &Value,
    modloader_json: Option<&Value>,
    paths: &LaunchPaths,
    loader_libs_path: &PathBuf,
    environment: &LaunchEnvironment,
) -> ClasspathData {
    let mut raw_libs = Vec::new();

    if let Some(loader_json) = modloader_json {
        let loader_libs = verify_libraries(
            loader_json,
            loader_libs_path,
            &LaunchContext {
                my_os: environment.my_os.clone(),
                my_arch: environment.my_arch.clone(),
                placeholders: HashMap::new(),
            },
        );

        raw_libs.extend(loader_libs);
    }

    let vanilla_libs = verify_libraries(
        version_json,
        &paths.libs,
        &LaunchContext {
            my_os: environment.my_os.clone(),
            my_arch: environment.my_arch.clone(),
            placeholders: HashMap::new(),
        },
    );

    raw_libs.extend(vanilla_libs);

    let mut seen_keys = std::collections::HashSet::new();
    let mut classpath_items = Vec::new();

    for lib in &raw_libs {
        if seen_keys.insert(lib.key.clone()) {
            classpath_items.push(lib.path.clone());
        }
    }

    if paths.client_jar.exists() {
        classpath_items.push(paths.client_jar.clone());
    }

    let separator = if cfg!(windows) { ";" } else { ":" };

    let classpath = classpath_items
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(separator);

    ClasspathData {
        classpath,
        separator: separator.to_string(),
    }
}
