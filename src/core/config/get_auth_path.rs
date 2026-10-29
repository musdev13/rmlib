use super::paths::Paths;

pub fn get_auth_path() -> String {
    Paths::load().expect("failed to load paths config").auth
}
