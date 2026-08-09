use std::process::Command;

use crate::core::run::build_command::build_command;

pub async fn run_client(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    spawn_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
) {
    println!(
        "{}: preparing launch sequence...",
        musutils::types::Status::Task.as_colored_str()
    );

    let command = match build_command(
        version_id,
        fabric_version,
        neoforge_version,
        ram_count,
        username,
        uuid,
        token,
        ely,
        betafix,
        versions_path,
        assets_path,
        libs_path,
        native_path,
        game_path,
        custom_jvm_args,
        custom_game_args,
    ) {
        Ok(command) => command,

        Err(e) => {
            println!(
                "{}: {}",
                musutils::types::Status::Err.as_colored_str(),
                e
            );
            return;
        }
    };

    let norm_spawn_path = musutils::fs::tilda_desir(spawn_path);

    println!(
        "{}: spawn dir - {}",
        musutils::types::Status::Inf.as_colored_str(),
        norm_spawn_path.display()
    );

    println!(
        "{}: spawning java process...",
        musutils::types::Status::Task.as_colored_str()
    );

    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(&norm_spawn_path)
        .spawn()
    {
        Ok(child) => child,

        Err(e) => {
            println!(
                "{}: failed to execute java process: {}",
                musutils::types::Status::Err.as_colored_str(),
                e
            );
            return;
        }
    };

    println!(
        "{}: instance started successfully! Press Ctrl+C to exit.",
        musutils::types::Status::Ok.as_colored_str()
    );

    match child.wait() {
        Ok(status) => {
            println!(
                "{}: game exited with status: {}",
                musutils::types::Status::Ok.as_colored_str(),
                status
            );
        }

        Err(e) => {
            println!(
                "{}: error waiting for game process: {}",
                musutils::types::Status::Err.as_colored_str(),
                e
            );
        }
    }
}
