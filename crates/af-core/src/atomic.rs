//! Atomic write workflow: temp file in same directory, flush/sync, replace.

use crate::errors::AfError;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AfError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| AfError::io(path, e))?;
    let mut tmp = NamedTempFile::new_in(parent).map_err(|e| AfError::io(path, e))?;
    tmp.write_all(bytes).map_err(|e| AfError::io(path, e))?;
    tmp.as_file().sync_all().map_err(|e| AfError::io(path, e))?;
    tmp.persist(path).map_err(|e| AfError::io(path, e.error))?;
    Ok(())
}

