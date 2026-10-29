use std::process::Command;

use crate::core::run::build_command::build_argv;

pub(crate) fn resolve_java_executable<'a>(custom: Option<&'a str>, fallback: &'a str) -> &'a str {
    match custom {
        Some(p) if !p.is_empty() => p,
        _ => fallback,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run_client(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    spawn_path: &str,
    java_path: Option<&str>,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
) {
    println!(
        "{}: preparing launch sequence...",
        musutils::types::Status::Task.as_colored_str()
    );

    let argv = match build_argv(
        version_id,
        fabric_version,
        neoforge_version,
        ram_count,
        username,
        uuid,
        xuid,
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
        false,
    ) {
        Ok(argv) => argv,
        Err(e) => {
            println!("{}: {}", musutils::types::Status::Err.as_colored_str(), e);
            return;
        }
    };

    if argv.is_empty() {
        println!(
            "{}: empty command",
            musutils::types::Status::Err.as_colored_str()
        );
        return;
    }

    let norm_spawn_path = musutils::fs::expand(spawn_path);

    println!(
        "{}: spawn dir - {}",
        musutils::types::Status::Inf.as_colored_str(),
        norm_spawn_path.display()
    );

    let java_executable = resolve_java_executable(java_path, &argv[0]);

    println!(
        "{}: spawning java process...",
        musutils::types::Status::Task.as_colored_str()
    );

    let mut child = match Command::new(java_executable)
        .args(&argv[1..])
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

#[cfg(test)]
mod tests {
    use super::resolve_java_executable;

    #[test]
    fn resolve_java_executable_none_returns_fallback() {
        assert_eq!(resolve_java_executable(None, "java"), "java");
    }

    #[test]
    fn resolve_java_executable_empty_returns_fallback() {
        assert_eq!(resolve_java_executable(Some(""), "java"), "java");
    }

    #[test]
    fn resolve_java_executable_some_returns_custom() {
        assert_eq!(
            resolve_java_executable(Some("/opt/jdk/bin/java"), "java"),
            "/opt/jdk/bin/java"
        );
    }
}
