use serde_json::Value;
use std::path::Path;

use crate::core::run::dto::{LaunchContext, ResolvedLibrary};
use crate::core::run::{check_rules, get_library_group_artifact};

pub fn verify_libraries(
    version_json: &Value,
    libs_path: &Path,
    context: &LaunchContext,
) -> Vec<ResolvedLibrary> {
    let mut classpath_entries = Vec::new();

    let os_str = context.my_os.as_str();
    let arch_str = context.my_arch.as_str();

    if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
        for library in libraries {
            let mut is_allowed = true;

            if let Some(rules) = library.get("rules").and_then(|r| r.as_array()) {
                is_allowed = check_rules(rules, context);
            }

            if !is_allowed {
                continue;
            }

            let name = library.get("name").and_then(|n| n.as_str()).unwrap_or("");

            if name.is_empty() {
                continue;
            }

            let mut rel_paths = Vec::new();

            let native_classifier = library
                .get("natives")
                .and_then(|n| n.get(&context.my_os))
                .and_then(|c| c.as_str())
                .map(|c| c.replace("${arch}", arch_str));

            if let Some(downloads) = library.get("downloads") {
                if let Some(ref classifier) = native_classifier
                    && let Some(classifier_obj) =
                        downloads.get("classifiers").and_then(|c| c.get(classifier))
                    && let Some(p) = classifier_obj.get("path").and_then(|v| v.as_str())
                {
                    rel_paths.push(p.to_string());
                }

                if let Some(artifact) = downloads.get("artifact")
                    && let Some(p) = artifact.get("path").and_then(|v| v.as_str())
                {
                    rel_paths.push(p.to_string());
                }

                if rel_paths.is_empty()
                    && let Some(classifiers) =
                        downloads.get("classifiers").and_then(|c| c.as_object())
                {
                    for (key, val) in classifiers {
                        let key_matches_os =
                            key.contains(os_str) || (os_str == "osx" && key.contains("macos"));

                        if key_matches_os && let Some(p) = val.get("path").and_then(|v| v.as_str())
                        {
                            rel_paths.push(p.to_string());
                        }
                    }
                }
            }

            if rel_paths.is_empty()
                && let Some(maven_path) = crate::core::types::build_maven_path(name, None)
            {
                rel_paths.push(maven_path);
            }

            let group_artifact = get_library_group_artifact(name);

            for rel_path in rel_paths {
                if rel_path.is_empty() {
                    continue;
                }

                let key = if let Some(ref classifier) = native_classifier {
                    format!("{}:{}", group_artifact, classifier)
                } else if name.contains("natives-") {
                    format!("{}:{}", group_artifact, rel_path)
                } else {
                    group_artifact.clone()
                };

                let full_lib_path = libs_path.join(&rel_path);

                if !full_lib_path.exists() {
                    println!(
                        "{}: Missing library -> {}",
                        musutils::types::Status::Warn.as_colored_str(),
                        full_lib_path.display()
                    );
                } else {
                    classpath_entries.push(ResolvedLibrary {
                        key,
                        path: full_lib_path,
                    });
                }
            }
        }
    }

    classpath_entries
}
