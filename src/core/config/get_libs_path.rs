use super::paths::Paths;

pub fn get_libs_path() -> String {
    Paths::load().expect("failed to load paths config").libs
}
