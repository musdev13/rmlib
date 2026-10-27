mod download_client_jar;
mod download_libraries;
mod extract_native_libraries;
#[allow(clippy::module_inception)]
pub mod install;
mod prepare_version_files;
mod process_assets;
mod resolve_installation_paths;
mod resolve_version_data;

pub use download_client_jar::download_client_jar;
pub use download_libraries::download_libraries;
pub use extract_native_libraries::extract_native_libraries;
pub use install::install;
pub use prepare_version_files::prepare_version_files;
pub use process_assets::process_assets;
pub use resolve_installation_paths::resolve_installation_paths;
pub use resolve_version_data::resolve_version_data;
