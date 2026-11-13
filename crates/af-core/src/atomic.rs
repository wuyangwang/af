//! Atomic write workflow: temp file in same directory, flush/sync, replace.

use crate::errors::AfError;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AfError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| AfError::Io(e.to_string()))?;
    let mut tmp = NamedTempFile::new_in(parent).map_err(|e| AfError::Io(e.to_string()))?;
    tmp.write_all(bytes).map_err(|e| AfError::Io(e.to_string()))?;
    tmp.as_file().sync_all().map_err(|e| AfError::Io(e.to_string()))?;
    tmp.persist(path).map_err(|e| AfError::Io(e.error.to_string()))?;
    Ok(())
}

