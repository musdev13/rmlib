use super::paths::Paths;

pub fn get_versions_path() -> String {
    Paths::load().expect("failed to load paths config").versions
}
