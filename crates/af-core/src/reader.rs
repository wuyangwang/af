//! File reading: full/range reads, metadata, binary detection.

use crate::encoding::{decode_to_string, Bom, DetectedEncoding};
use crate::errors::AfError;
use crate::hash::sha256_hex;
use crate::newline::{detect_newline, NewlineStyle};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileMeta {
    pub encoding: String,
    pub bom: bool,
    pub eol: String,
    pub size: u64,
    pub sha256: String,
    pub mixed_newlines: bool,
}

#[derive(Debug, Clone)]
pub struct ReadResult {
    pub content: String,
    pub meta: FileMeta,
    pub detected: DetectedEncoding,
    pub newline: NewlineStyle,
}

pub fn looks_binary(bytes: &[u8], bom: Bom) -> bool {
    if matches!(bom, Bom::Utf16Le | Bom::Utf16Be) {
        return false;
    }
    bytes.contains(&0)
}

pub fn read_text_file(path: &Path, explicit_encoding: Option<&str>) -> Result<ReadResult, AfError> {
    let bytes = fs::read(path).map_err(|e| AfError::Io(format!("{}: {e}", path.display())))?;
    let size = bytes.len() as u64;
    let (bom, _) = crate::encoding::detect_bom(&bytes);
    if looks_binary(&bytes, bom) && explicit_encoding.is_none() {
        return Err(AfError::Encoding(format!(
            "binary file detected: {}",
            path.display()
        )));
    }
    let (content, detected) = decode_to_string(&bytes, explicit_encoding)?;
    let newline = detect_newline(&content);
    let meta = FileMeta {
        encoding: detected.label.clone(),
        bom: !matches!(detected.bom, Bom::None),
        eol: newline.as_str().to_string(),
        size,
        sha256: sha256_hex(&bytes),
        mixed_newlines: matches!(newline, NewlineStyle::Mixed),
    };
    Ok(ReadResult {
        content,
        meta,
        detected,
        newline,
    })
}

