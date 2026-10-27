use super::paths::Paths;

pub fn init(force: bool) {
    println!("initializing...");
    if force {
        println!("!!! FORCED !!!");
    }
    match Paths::init(force) {
        Ok(()) => {
            println!(
                "done :3\n\n!!! check your ~/.config or %AppData% directory for rml configuration !!!"
            );
        }
        Err(e) => {
            eprintln!(
                "{}: failed to init config: {}",
                musutils::types::Status::Err.as_colored_str(),
                e
            );
        }
    }
}
