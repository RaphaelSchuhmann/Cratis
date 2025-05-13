use std::path::Path;
use crate::error::{CratisError, CratisResult};

pub fn ensure_path_exists(path: &Path) -> CratisResult<()> {
    if !path.exists() {
        let path_str = path.to_string_lossy().into_owned();
        return Err(CratisError::InvalidPath(path_str));
    }
    Ok(())
}

