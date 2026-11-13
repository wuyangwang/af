use crate::errors::AfError;
use std::path::{Component, Path};

/// Basic path safety: reject paths with parent-dir traversal components.
pub fn ensure_safe_path(path: &Path) -> Result<(), AfError> {
    for c in path.components() {
        if matches!(c, Component::ParentDir) {
            return Err(AfError::safety(
                "PATH_TRAVERSAL",
                format!("path traversal is not allowed: {}", path.display()),
            ));
        }
    }
    Ok(())
}
