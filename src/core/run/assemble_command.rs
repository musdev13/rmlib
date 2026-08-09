use crate::core::run::build_arguments::LaunchArguments;
use crate::core::run::dto::LaunchContext;
use crate::core::run::replace_placeholders::replace_placeholders;

pub fn assemble_command(
    main_class: &str,
    arguments: LaunchArguments,
    context: &LaunchContext,
) -> String {
    let final_jvm_args: Vec<String> = arguments
        .jvm_args
        .into_iter()
        .map(|arg| {
            replace_placeholders(
                &arg,
                &context.placeholders,
            )
        })
        .collect();

    let final_game_args: Vec<String> = arguments
        .game_args
        .into_iter()
        .map(|arg| {
            replace_placeholders(
                &arg,
                &context.placeholders,
            )
        })
        .collect();

    let mut command = Vec::new();

    command.push("java".to_string());
    command.extend(final_jvm_args);
    command.push(main_class.to_string());
    command.extend(final_game_args);

    command
        .into_iter()
        .map(|arg| {
            if arg.contains(' ') {
                format!("\"{}\"", arg)
            } else {
                arg
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
