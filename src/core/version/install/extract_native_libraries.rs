use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use musutils;

pub fn extract_native_libraries(libs_path: &Path) {
    let natives_target_path = libs_path.join("natives");
    fs::create_dir_all(&natives_target_path).ok();

    println!(
        "{}: extracting native libraries to {}...",
        musutils::types::Status::Task.as_colored_str(),
        natives_target_path.display()
    );

    let target_ext = match musutils::os::get_os() {
        musutils::os::OS::Windows => ".dll",
        musutils::os::OS::Linux => ".so",
        musutils::os::OS::MacOs => ".dylib",
        _ => "",
    };

    fn scan_dir(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
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
    let _ = scan_dir(libs_path, &mut jar_files);

    for jar_path in jar_files {
        let jar_name = jar_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if !jar_name.contains("native") && !jar_name.contains("platform") {
            continue;
        }

        if let Ok(file) = fs::File::open(&jar_path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        let name = file.name();

                        if name.contains("META-INF") || file.is_dir() {
                            continue;
                        }

                        if !target_ext.is_empty() && name.ends_with(target_ext) {
                            if let Some(file_name) = Path::new(name).file_name() {
                                let out_path = natives_target_path.join(file_name);

                                if let Ok(mut out_file) = fs::File::create(&out_path) {
                                    io::copy(&mut file, &mut out_file).ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "{}: natives extracted successfully!",
        musutils::types::Status::Ok.as_colored_str()
    );
}
