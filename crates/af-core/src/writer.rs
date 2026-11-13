//! File creation and overwrite with preserve/normalize/strict policies.

use crate::atomic::atomic_write;
use crate::encoding::{encode_from_string, Bom};
use crate::errors::AfError;
use crate::hash::sha256_hex;
use crate::newline::{detect_newline, normalize_newlines, NewlineStyle};
use crate::reader::{read_text_file, FileMeta};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Preserve,
    Normalize,
    Strict,
}

#[derive(Debug, Clone)]
pub struct WriteOutcome {
    pub meta: FileMeta,
    pub bytes_written: usize,
}

pub fn write_new_file(path: &Path, text: &str) -> Result<WriteOutcome, AfError> {
    // New files: UTF-8 no BOM + LF
    let normalized = normalize_newlines(text, NewlineStyle::Lf);
    let bytes = encode_from_string(&normalized, "utf-8", Bom::None)?;
    atomic_write(path, &bytes)?;
    let newline = detect_newline(&normalized);
    Ok(WriteOutcome {
        bytes_written: bytes.len(),
        meta: FileMeta {
            encoding: "utf-8".into(),
            bom: false,
            eol: newline.as_str().into(),
            size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            mixed_newlines: false,
        },
    })
}

pub fn write_preserving(path: &Path, text: &str, mode: EditMode) -> Result<WriteOutcome, AfError> {
    if !path.exists() {
        return write_new_file(path, text);
    }
    let existing = read_text_file(path, None)?;
    let target_eol = match mode {
        EditMode::Normalize => NewlineStyle::Lf,
        _ => existing.newline,
    };
    if matches!(mode, EditMode::Strict) && matches!(existing.newline, NewlineStyle::Mixed) {
        return Err(AfError::Safety("mixed newlines in strict mode".into()));
    }
    let content = normalize_newlines(text, target_eol);
    // Preserve final newline presence is not yet fully enforced; content is normalized per policy.

    let (label, bom) = match mode {
        EditMode::Normalize => ("utf-8".to_string(), Bom::None),
        _ => (existing.detected.label.clone(), existing.detected.bom),
    };
    if matches!(mode, EditMode::Strict) {
        // strict: require already utf-8 no bom + lf? simple check
        if !(label == "utf-8" && matches!(bom, Bom::None) && matches!(target_eol, NewlineStyle::Lf | NewlineStyle::None)) {
            return Err(AfError::Safety("file does not meet strict target format".into()));
        }
    }
    let bytes = encode_from_string(&content, &label, bom)?;
    atomic_write(path, &bytes)?;
    let newline = detect_newline(&content);
    Ok(WriteOutcome {
        bytes_written: bytes.len(),
        meta: FileMeta {
            encoding: label,
            bom: !matches!(bom, Bom::None),
            eol: newline.as_str().into(),
            size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            mixed_newlines: matches!(newline, NewlineStyle::Mixed),
        },
    })
}

