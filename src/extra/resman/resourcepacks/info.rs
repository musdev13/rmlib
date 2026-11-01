use std::path::Path;

use super::dto::ResourcepackInfo;
use super::error::RpError;
use super::parse::analyze_zip;

pub fn info_from_file(path: &Path) -> Result<ResourcepackInfo, RpError> {
    if !path.exists() {
        return Err(RpError::NotFound(format!(
            "file not found: {}",
            path.display()
        )));
    }
    analyze_zip(path)
}
