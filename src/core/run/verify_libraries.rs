use serde_json::Value;
use std::path::Path;

use crate::core::run::dto::{LaunchContext, ResolvedLibrary};
use crate::core::run::{check_rules, get_library_group_artifact};

fn arch_matches_classifier(classifier: &str, my_arch: &str) -> bool {
    let c = classifier.to_ascii_lowercase();
    let has_arm64 = c.contains("arm64") || c.contains("aarch64");
    let has_arm32 = c.contains("arm32") || (c.contains("arm") && !has_arm64);
    let has_x86 = c.contains("x86_64") || c.contains("amd64") || c.contains("64");
    let has_32 = c.contains("x86") && !c.contains("x86_64")
        || c.contains("32")
        || c.contains("i386")
        || c.contains("i686");

    match my_arch {
        "64" => !has_arm64 && !has_arm32 && !has_32,
        "32" => !has_arm64 && !has_arm32 && !has_x86,
        "arm64" => has_arm64,
        "arm32" => has_arm32,
        _ => true,
    }
}

pub fn verify_libraries(
    version_json: &Value,
    libs_path: &Path,
    context: &LaunchContext,
) -> Vec<ResolvedLibrary> {
    let mut classpath_entries = Vec::new();

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

                let is_native = native_classifier.is_some() || name.contains(":natives-");
                if is_native
                    && let Some(classifier) = rel_path.rsplit('/').next()
                    && let Some(stem) = classifier.strip_suffix(".jar")
                    && !arch_matches_classifier(stem, &context.my_arch)
                {
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
