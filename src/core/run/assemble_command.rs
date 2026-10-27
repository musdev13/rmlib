use crate::core::run::build_arguments::LaunchArguments;
use crate::core::run::dto::LaunchContext;
use crate::core::run::replace_placeholders::replace_placeholders;

pub fn assemble_argv(
    main_class: &str,
    arguments: LaunchArguments,
    context: &LaunchContext,
) -> Vec<String> {
    let final_jvm_args: Vec<String> = arguments
        .jvm_args
        .into_iter()
        .map(|arg| replace_placeholders(&arg, &context.placeholders))
        .collect();

    let final_game_args: Vec<String> = arguments
        .game_args
        .into_iter()
        .map(|arg| replace_placeholders(&arg, &context.placeholders))
        .collect();

    let mut argv = Vec::with_capacity(final_jvm_args.len() + final_game_args.len() + 2);
    argv.push("java".to_string());
    argv.extend(final_jvm_args);
    argv.push(main_class.to_string());
    argv.extend(final_game_args);
    argv
}

pub fn assemble_command(
    main_class: &str,
    arguments: LaunchArguments,
    context: &LaunchContext,
) -> String {
    assemble_argv(main_class, arguments, context)
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
