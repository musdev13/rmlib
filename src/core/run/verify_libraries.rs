use std::path::Path;
use serde_json::Value;

use crate::core::run::dto::{LaunchContext, ResolvedLibrary};
use crate::core::run::{check_rules, get_library_group_artifact};

pub fn verify_libraries(
    version_json: &Value,
    libs_path: &Path,
    context: &LaunchContext,
) -> Vec<ResolvedLibrary> {
    let mut classpath_entries = Vec::new();

    let os_native_key = match context.my_os.as_str() {
        "windows" => "natives-windows",
        "linux" => "natives-linux",
        "osx" => "natives-macos",
        _ => "unknown",
    };

    let build_maven_path = |name: &str| -> Option<String> {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 { return None; }
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        if parts.len() == 4 {
            Some(format!("{}/{}/{}/{}-{}-{}.jar", group, artifact, version, artifact, version, parts[3]))
        } else {
            Some(format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version))
        }
    };

    if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
        for library in libraries {
            let mut is_allowed = true;
            if let Some(rules) = library.get("rules").and_then(|r| r.as_array()) {
                is_allowed = check_rules(rules, context);
            }

            if !is_allowed { continue; }

            let name = library.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.is_empty() { continue; }

            if name.contains(":natives-") && !name.contains(os_native_key) {
                continue;
            }

            let mut rel_path = String::new();
            let native_classifier = library.get("natives")
                .and_then(|n| n.get(&context.my_os))
                .and_then(|c| c.as_str())
                .map(|c| c.replace("${arch}", &context.my_arch));

            if let Some(downloads) = library.get("downloads") {
                if let Some(ref classifier) = native_classifier {
                    if let Some(classifier_obj) = downloads.get("classifiers").and_then(|c| c.get(classifier)) {
                        rel_path = classifier_obj.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    }
                } else if let Some(artifact) = downloads.get("artifact") {
                    rel_path = artifact.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                }
            }

            if rel_path.is_empty() {
                if let Some(maven_path) = build_maven_path(name) {
                    rel_path = maven_path;
                }
            }

            if rel_path.is_empty() { continue; }

            let group_artifact = get_library_group_artifact(name);
            let key = if let Some(ref classifier) = native_classifier {
                format!("{}:{}", group_artifact, classifier)
            } else {
                group_artifact
            };

            let full_lib_path = libs_path.join(&rel_path);

            if !full_lib_path.exists() {
                println!(
                    "{}: Missing library -> {}",
                    musutils::types::Status::Warn.as_colored_str(),
                    full_lib_path.display()
                );
            } else {
                classpath_entries.push(ResolvedLibrary { key, path: full_lib_path });
            }
        }
    }
    classpath_entries
}
