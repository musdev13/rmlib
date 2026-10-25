use std::path::PathBuf;
use std::process::Command;

pub async fn install(
    version_id: String,
    neoforge_version: String,
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
) {
    let line = musutils::types::line::draw_colored('=', 35, musutils::color::Colors::Yellow);

    println!(
        "{}: soft installing base version for NeoForge...",
        musutils::types::Status::Task.as_colored_str()
    );

    let soft = true;
    let iclient = true;
    let ilibs = true;
    let iassets = true;
    let alibs = 150;
    let aassets = 350;
    let json_path = None;

    crate::core::version::install::install(
        Some(version_id.clone()),
        json_path,
        versions_path_option.clone(),
        libs_path_option.clone(),
        assets_path_option.clone(),
        soft,
        iclient,
        ilibs,
        iassets,
        alibs,
        aassets,
    )
    .await;

    println!("{}", line.clone());
    println!(
        "{}: base version for NeoForge successfully soft-installed!",
        musutils::types::Status::Ok.as_colored_str()
    );
    println!("{}", line.clone());

    let (_, _, _, versions_path, libs_path, _) =
        crate::core::version::install::resolve_installation_paths(
            versions_path_option,
            libs_path_option,
            assets_path_option,
        );

    println!("{}", line.clone());

    let temp_dir = versions_path
        .join("modloaders")
        .join("neoforge")
        .join("temp");

    println!(
        "{}: creating NeoForge temp directory...",
        musutils::types::Status::Task.as_colored_str()
    );
    musutils::fs::new_dir(temp_dir.clone());
    println!(
        "{}: temp directory created!",
        musutils::types::Status::Ok.as_colored_str()
    );

    println!("{}", line.clone());

    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        neoforge_version, neoforge_version
    );

    println!(
        "{}: downloading NeoForge installer for `{}`...",
        musutils::types::Status::Task.as_colored_str(),
        neoforge_version
    );

    let installer_bytes = match musutils::http::get_file(installer_url).await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!(
                "{}: failed to download NeoForge installer! Cleaning up...",
                musutils::types::Status::Err.as_colored_str()
            );
            let _ = musutils::fs::remove_dir_all(&temp_dir);
            return;
        }
    };

    let installer_filename = format!("neoforge-{}-installer.jar", neoforge_version);
    let installer_path = temp_dir.join(&installer_filename);

    if musutils::fs::write(installer_path.clone(), installer_bytes).is_err() {
        println!(
            "{}: failed to save NeoForge installer to disk! Cleaning up...",
            musutils::types::Status::Err.as_colored_str()
        );
        let _ = musutils::fs::remove_dir_all(&temp_dir);
        return;
    }

    println!(
        "{}: NeoForge installer downloaded successfully!",
        musutils::types::Status::Ok.as_colored_str()
    );

    println!("{}", line.clone());

    println!(
        "{}: preparing fakemc folder...",
        musutils::types::Status::Task.as_colored_str()
    );

    let fakemc_dir = temp_dir.join("fakemc");

    println!(
        "{}: creating fakemc directory...",
        musutils::types::Status::Inf.as_colored_str()
    );
    musutils::fs::new_dir(fakemc_dir.clone());

    let profiles_json_path = fakemc_dir.join("launcher_profiles.json");

    println!(
        "{}: creating launcher profile json...",
        musutils::types::Status::Inf.as_colored_str()
    );
    if musutils::fs::write(profiles_json_path, "{}").is_err() {
        println!(
            "{}: failed to write launcher_profiles.json! Cleaning up...",
            musutils::types::Status::Err.as_colored_str()
        );
        let _ = musutils::fs::remove_dir_all(&temp_dir);
        return;
    }

    println!(
        "{}: fakemc folder prepared successfully!",
        musutils::types::Status::Ok.as_colored_str()
    );

    println!("{}", line.clone());

    println!(
        "{}: running NeoForge installer...",
        musutils::types::Status::Task.as_colored_str()
    );

    let status = Command::new("java")
        .current_dir(&temp_dir)
        .arg("-jar")
        .arg(&installer_filename)
        .arg("--install-client")
        .arg("./fakemc")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!(
                "{}: NeoForge installer executed successfully!",
                musutils::types::Status::Ok.as_colored_str()
            );
        }
        _ => {
            println!(
                "{}: NeoForge installation failed! Cleaning up...",
                musutils::types::Status::Err.as_colored_str()
            );
            let _ = musutils::fs::remove_dir_all(&temp_dir);
            return;
        }
    }

    println!("{}", line.clone());

    println!(
        "{}: moving libraries to destination...",
        musutils::types::Status::Task.as_colored_str()
    );
    let source_libs = fakemc_dir.join("libraries");
    let target_libs = libs_path.join("modloaders").join("neoforge");

    musutils::fs::new_dir(target_libs.clone());

    if musutils::fs::copy_dir_all(source_libs, target_libs).is_err() {
        println!(
            "{}: failed to move NeoForge libraries! Cleaning up...",
            musutils::types::Status::Err.as_colored_str()
        );
        let _ = musutils::fs::remove_dir_all(&temp_dir);
        return;
    }

    println!(
        "{}: moving version json...",
        musutils::types::Status::Task.as_colored_str()
    );
    let source_version_json = fakemc_dir
        .join("versions")
        .join(format!("neoforge-{}", neoforge_version))
        .join(format!("neoforge-{}.json", neoforge_version));

    let target_version_dir = versions_path
        .join("modloaders")
        .join("neoforge")
        .join(&version_id)
        .join(&neoforge_version);

    musutils::fs::new_dir(target_version_dir.clone());

    let target_version_json = target_version_dir.join(format!("{}.json", neoforge_version));

    if musutils::fs::copy(source_version_json, target_version_json).is_err() {
        println!(
            "{}: failed to move NeoForge version json! Cleaning up...",
            musutils::types::Status::Err.as_colored_str()
        );
        let _ = musutils::fs::remove_dir_all(&temp_dir);
        return;
    }

    println!(
        "{}: cleaning up temp directory...",
        musutils::types::Status::Task.as_colored_str()
    );
    let _ = musutils::fs::remove_dir_all(&temp_dir);

    println!("{}", line.clone());
    println!(
        "{}: NeoForge installation completed successfully!",
        musutils::types::Status::Ok.as_colored_str()
    );
    println!("{}", line.clone());
}
