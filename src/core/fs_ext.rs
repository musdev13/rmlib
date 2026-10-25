use std::path::Path;

pub fn ensure_dir<P: AsRef<Path>>(path: P) {
    if let Err(e) = musutils::fs::create_dir_all(&path) {
        eprintln!(
            "{}: failed to create dir `{}`: {}",
            musutils::types::Status::Err.as_colored_str(),
            path.as_ref().display(),
            e
        );
    }
}
