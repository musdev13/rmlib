use std::path::PathBuf;

use crate::core;

pub async fn install(version_id: String, versions_path_option: Option<PathBuf>, libs_path_option: Option<PathBuf>, assets_path_option: Option<PathBuf>){
    let line = musutils::types::line::draw_colored('=', 35, musutils::color::Colors::Yellow);

    println!("{}: fetching `{}`...", musutils::types::Status::Task.as_colored_str(), &version_id);
    
    let version_json_str = core::version::get_version_json(version_id.clone()).await;

    println!("{}\n{}\n{}",line.clone(), version_json_str.clone(), line.clone());
    println!("{}: version `{}` found!", musutils::types::Status::Ok.as_colored_str(), version_id);

    println!("{}", line.clone());
    

    println!("{}: resolving paths...", musutils::types::Status::Task.as_colored_str());
    let mut is_custom_versions_path = false;
    let mut is_custom_libs_path = false;
    let mut is_custom_assets_path = false;
    let versions_path = musutils::fs::tilda_desir(
        musutils::types::deoption(
            versions_path_option, 
            || PathBuf::from(core::config::get_versions_path()),
            &mut is_custom_versions_path
        )
    );
    let libs_path = musutils::fs::tilda_desir(
        musutils::types::deoption(
            libs_path_option, 
            || PathBuf::from(core::config::get_libs_path()),
            &mut is_custom_libs_path
        )
    );
    let assets_path = musutils::fs::tilda_desir(
        musutils::types::deoption(
            assets_path_option, 
            || PathBuf::from(core::config::get_assets_path()),
            &mut is_custom_assets_path
        )
    );
    {
        let star = musutils::color::color_str("*", musutils::color::Colors::Orange); 
        println!("{}: got next paths:\n{}\n{}\n{}", musutils::types::Status::Ok.as_colored_str(),
            format!("{} {}", if is_custom_versions_path {star.clone()} else {"-".to_string()}, versions_path.clone().display() ),
            format!("{} {}", if is_custom_libs_path {star.clone()} else {"-".to_string()}, libs_path.clone().display() ),
            format!("{} {}", if is_custom_assets_path {star.clone()} else {"-".to_string()}, assets_path.clone().display() ),
        );
    }

    print!("{}: creating directories... ",musutils::types::Status::Task.as_colored_str());
    musutils::fs::new_dir(versions_path.clone());
    musutils::fs::new_dir(libs_path.clone());
    musutils::fs::new_dir(assets_path.clone());
    let version_path = versions_path.clone().join(version_id.clone());
    musutils::fs::new_dir(version_path.clone());
    println!("{}",musutils::types::Status::Ok.as_colored_str());
    
    println!("{}", line.clone());
    print!("{}: writing `{}.json`... ", musutils::types::Status::Task.as_colored_str(), version_id.clone());
    musutils::fs::write(version_path.clone().join(format!("{}{}", version_id.clone(), ".json")), version_json_str.clone()).expect(&format!("{}: can't write {}.json", musutils::types::Status::Err.as_colored_str(),version_id.clone()));
    println!("{}", musutils::types::Status::Ok.as_colored_str());
    
    

    println!("{}", line.clone());
    println!("{}: parsing and downloading client jar...", musutils::types::Status::Task.as_colored_str());
    
    let version_json: serde_json::Value = serde_json::from_str(&version_json_str)
        .expect(&format!("{}: can't parse client jar url", musutils::types::Status::Err.as_colored_str()));
    let downloads = version_json.get("downloads")
        .expect(&format!("{}: 'downloads' field is missing", musutils::types::Status::Err.as_colored_str()));
    let client = downloads.get("client")
        .expect(&format!("{}: 'client' field is missing in downloads", musutils::types::Status::Err.as_colored_str()));
    let client_sha1 = client.get("sha1")
        .and_then(|v| v.as_str())
        .expect(&format!("{}: 'sha1' field is missing or not a string", musutils::types::Status::Err.as_colored_str()))
        .to_string();
    let client_url = client.get("url")
        .and_then(|v| v.as_str())
        .expect(&format!("{}: 'url' field is missing or not a string", musutils::types::Status::Err.as_colored_str()))
        .to_string();

    let jar_path = version_path.clone().join(format!("{}.jar", version_id.clone()));

    musutils::http::download_to_sha1(client_url, jar_path, client_sha1).await
        .expect(&format!("{}: failed to download client jar", musutils::types::Status::Err.as_colored_str()));
    
    println!("{}: we have client.jar! :3",musutils::types::Status::Ok.as_colored_str());

    

    println!("{}", line.clone());
    println!("{}: downloading libraries...", musutils::types::Status::Task.as_colored_str());
    
    let mut libs_downloader = musutils::http::AsyncDownloader::new(50, musutils::http::async_downloader::HashAlgo::Sha1);

    let my_os = match musutils::os::get_os() {
        musutils::os::OS::Windows => "windows",
        musutils::os::OS::Linux => "linux",
        musutils::os::OS::MacOs => "osx",
        musutils::os::OS::Unknown => "unknown",
    };

    let my_arch = musutils::os::get_arch();
    
    let my_arch_str = match my_arch {
        musutils::os::Arch::X64 => "64",
        musutils::os::Arch::X86 => "32",
        musutils::os::Arch::Arm64 => "arm64",
        musutils::os::Arch::Arm => "arm32",
        musutils::os::Arch::Unknown => "unknown",
    };

    let (os_native_key, arch_native_key) = match musutils::os::get_os() {
        musutils::os::OS::Windows => {
            match my_arch {
                musutils::os::Arch::Arm64 => ("natives-windows", Some("arm64")),
                musutils::os::Arch::X86 => ("natives-windows", Some("x86")),
                _ => ("natives-windows", None),
            }
        },
        musutils::os::OS::Linux => {
            match my_arch {
                musutils::os::Arch::Arm64 => ("natives-linux", Some("arm64")),
                _ => ("natives-linux", None),
            }
        },
        musutils::os::OS::MacOs => {
            match my_arch {
                musutils::os::Arch::Arm64 => ("natives-macos", Some("arm64")),
                _ => ("natives-macos", None),
            }
        },
        _ => ("unknown", None),
    };

    let build_maven_path = |name: &str, classifier: Option<&str>| -> Option<String> {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 { return None; }
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        
        if let Some(cls) = classifier {
            Some(format!("{}/{}/{}/{}-{}-{}.jar", group, artifact, version, artifact, version, cls))
        } else if parts.len() == 4 {
            Some(format!("{}/{}/{}/{}-{}-{}.jar", group, artifact, version, artifact, version, parts[3]))
        } else {
            Some(format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version))
        }
    };

    if let Some(libraries) = version_json.get("libraries").and_then(|l| l.as_array()) {
        for library in libraries {
            let mut is_allowed = true;

            if let Some(rules) = library.get("rules").and_then(|r| r.as_array()) {
                is_allowed = false;

                for rule in rules {
                    let action = rule.get("action").and_then(|a| a.as_str()).unwrap_or("allow");
                    let is_allow_action = action == "allow";
                
                    if let Some(os_filter) = rule.get("os") {
                        if let Some(os_name) = os_filter.get("name").and_then(|n| n.as_str()) {
                            if os_name == my_os {
                                is_allowed = is_allow_action;
                            }
                        }
                    } else {
                        is_allowed = is_allow_action;
                    }
                }
            }

            if !is_allowed {
                continue;
            }

            let name = library.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if name.is_empty() { continue; }

            if name.contains(":natives-") {
                if !name.contains(os_native_key) {
                    continue;
                }
                if let Some(arch_key) = arch_native_key {
                    if !name.contains(arch_key) { continue; }
                } else {
                    if name.contains("arm64") || name.contains("x86") || name.contains("amd64") || name.contains("-32") {
                        continue;
                    }
                }
            }

            let mut final_url = String::new();
            let mut final_path = String::new();
            let mut final_sha1: Option<String> = None;

            if let Some(downloads) = library.get("downloads") {
                let native_classifier = library.get("natives")
                    .and_then(|n| n.get(my_os))
                    .and_then(|c| c.as_str())
                    .map(|c| c.replace("${arch}", my_arch_str));

                if let Some(ref classifier) = native_classifier {
                    if let Some(classifier_obj) = downloads.get("classifiers").and_then(|c| c.get(classifier)) {
                        final_url = classifier_obj.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        final_path = classifier_obj.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        final_sha1 = classifier_obj.get("sha1").and_then(|v| v.as_str()).map(|s| s.to_string());
                    }
                } else if let Some(artifact) = downloads.get("artifact") {
                    final_url = artifact.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    final_path = artifact.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    final_sha1 = artifact.get("sha1").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
            } else {
                let mut ancient_classifier: Option<String> = None;

                if let Some(natives_obj) = library.get("natives") {
                    if let Some(cls) = natives_obj.get(my_os).and_then(|c| c.as_str()) {
                        ancient_classifier = Some(cls.replace("${arch}", my_arch_str));
                    } else {
                        continue;
                    }
                }

                if let Some(maven_path) = build_maven_path(name, ancient_classifier.as_deref()) {
                    final_path = maven_path.clone();
                    let base_url = library.get("url").and_then(|u| u.as_str()).unwrap_or("https://libraries.minecraft.net/");
                    final_url = format!("{}{}", base_url, maven_path);
                    final_sha1 = None;
                }
            }

            if final_url.is_empty() || final_path.is_empty() {
                continue;
            }

            let lib_path = libs_path.join(&final_path);

            println!(
                "{}: Adding library to queue -> {}",
                musutils::types::Status::Inf.as_colored_str(),
                final_path
            );
            
            libs_downloader.push(
                final_url,
                lib_path,
                final_sha1,
                Some(format!("{}: {{0}} downloaded", musutils::types::Status::Ok.as_colored_str())),
            );
        }
    }

    println!("{}: waiting downloading tasks", musutils::types::Status::Task.as_colored_str());
    libs_downloader.join().await.expect(&format!("{}: something happened with libs_downloader.. hehe... sowwy :3...", musutils::types::Status::Err.as_colored_str()));
    println!("{}: libs downloaded", musutils::types::Status::Ok.as_colored_str());

    let natives_target_path = libs_path.join("natives");
    std::fs::create_dir_all(&natives_target_path).ok();

    println!("{}: extracting native libraries to {}...", musutils::types::Status::Task.as_colored_str(), natives_target_path.display());

    let target_ext = match musutils::os::get_os() {
        musutils::os::OS::Windows => ".dll",
        musutils::os::OS::Linux => ".so",
        musutils::os::OS::MacOs => ".dylib",
        _ => "",
    };

    fn scan_dir(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    let mut jar_files = Vec::new();
    let _ = scan_dir(&libs_path, &mut jar_files);

    for jar_path in jar_files {
        let jar_name = jar_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if !jar_name.contains("native") && !jar_name.contains("platform") {
            continue;
        }

        if let Ok(file) = std::fs::File::open(&jar_path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        let name = file.name();

                        if name.contains("META-INF") || file.is_dir() {
                            continue;
                        }

                        if !target_ext.is_empty() && name.ends_with(target_ext) {
                            if let Some(file_name) = std::path::Path::new(name).file_name() {
                                let out_path = natives_target_path.join(file_name);
                                
                                if let Ok(mut out_file) = std::fs::File::create(&out_path) {
                                    std::io::copy(&mut file, &mut out_file).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("{}: natives extracted successfully!", musutils::types::Status::Ok.as_colored_str());
    println!("{}", line.clone());
}
