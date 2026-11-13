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
pub struct WriteOptions {
    pub mode: EditMode,
    pub encoding: Option<String>,
    pub bom: Option<bool>,
    pub eol: Option<NewlineStyle>,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            mode: EditMode::Preserve,
            encoding: None,
            bom: None,
            eol: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteOutcome {
    pub meta: FileMeta,
    pub bytes_written: usize,
    pub content: String,
}

pub fn write_new_file(path: &Path, text: &str) -> Result<WriteOutcome, AfError> {
    let outcome = plan_write(path, text, &WriteOptions::default())?;
    atomic_write(path, &outcome.encoded_bytes)?;
    Ok(WriteOutcome {
        bytes_written: outcome.encoded_bytes.len(),
        meta: outcome.meta,
        content: outcome.content,
    })
}

#[derive(Debug, Clone)]
pub struct WritePlan {
    pub meta: FileMeta,
    pub encoded_bytes: Vec<u8>,
    pub content: String,
}

pub fn plan_write(path: &Path, text: &str, opts: &WriteOptions) -> Result<WritePlan, AfError> {
    if !path.exists() {
        let target_eol = opts.eol.unwrap_or(NewlineStyle::Lf);
        let content = normalize_newlines(text, target_eol);
        let label = opts.encoding.clone().unwrap_or_else(|| "utf-8".to_string()).to_ascii_lowercase();
        let bom = match opts.bom {
            Some(true) if label.starts_with("utf-8") => Bom::Utf8,
            Some(true) if label.starts_with("utf-16le") => Bom::Utf16Le,
            Some(true) if label.starts_with("utf-16be") => Bom::Utf16Be,
            _ => Bom::None,
        };
        let bytes = encode_from_string(&content, &label, bom)?;
        let newline = detect_newline(&content);
        return Ok(WritePlan {
            encoded_bytes: bytes.clone(),
            content: content.clone(),
            meta: FileMeta {
                encoding: label,
                bom: !matches!(bom, Bom::None),
                eol: newline.as_str().into(),
                size: bytes.len() as u64,
                sha256: sha256_hex(&bytes),
                mixed_newlines: matches!(newline, NewlineStyle::Mixed),
            },
        });
    }

    let existing = read_text_file(path, opts.encoding.as_deref().or(None))?;
    if matches!(opts.mode, EditMode::Preserve)
        && matches!(existing.newline, NewlineStyle::Mixed)
        && opts.eol.is_none()
    {
        return Err(AfError::Safety {
            code: "MIXED_NEWLINE".into(),
            message: "mixed newline file: refuse to modify in preserve mode without explicit --eol/--normalize".into(),
            path: Some(path.display().to_string()),
        });
    }

    let target_eol = match opts.mode {
        EditMode::Normalize => NewlineStyle::Lf,
        _ => opts.eol.unwrap_or(existing.newline),
    };

    if matches!(opts.mode, EditMode::Strict) && matches!(existing.newline, NewlineStyle::Mixed) {
        return Err(AfError::Safety {
            code: "STRICT_FORMAT_VIOLATION".into(),
            message: "mixed newlines in strict mode".into(),
            path: Some(path.display().to_string()),
        });
    }

    let content = normalize_newlines(text, target_eol);
    let mut label = match opts.mode {
        EditMode::Normalize => "utf-8".to_string(),
        _ => opts.encoding.clone().unwrap_or_else(|| existing.detected.label.clone()),
    };
    label = label.to_ascii_lowercase();

    let bom = match opts.mode {
        EditMode::Normalize => Bom::None,
        _ => match opts.bom {
            Some(true) if label.starts_with("utf-8") => Bom::Utf8,
            Some(true) if label.starts_with("utf-16le") => Bom::Utf16Le,
            Some(true) if label.starts_with("utf-16be") => Bom::Utf16Be,
            Some(false) => Bom::None,
            Some(true) => existing.detected.bom,
            None => existing.detected.bom,
        },
    };

    if matches!(opts.mode, EditMode::Strict) {
        let ok = label == "utf-8"
            && matches!(bom, Bom::None)
            && matches!(target_eol, NewlineStyle::Lf | NewlineStyle::None);
        if !ok {
            return Err(AfError::Safety {
                code: "STRICT_FORMAT_VIOLATION".into(),
                message: "file does not meet strict target format (expected utf-8, no BOM, lf)".into(),
                path: Some(path.display().to_string()),
            });
        }
    }

    let bytes = encode_from_string(&content, &label, bom)?;
    let newline = detect_newline(&content);
    Ok(WritePlan {
        encoded_bytes: bytes.clone(),
        content: content.clone(),
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

pub fn write_with_options(path: &Path, text: &str, opts: &WriteOptions) -> Result<WriteOutcome, AfError> {
    let plan = plan_write(path, text, opts)?;
    atomic_write(path, &plan.encoded_bytes)?;
    Ok(WriteOutcome {
        bytes_written: plan.encoded_bytes.len(),
        meta: plan.meta,
        content: plan.content,
    })
}

pub fn write_preserving(path: &Path, text: &str, mode: EditMode) -> Result<WriteOutcome, AfError> {
    write_with_options(path, text, &WriteOptions { mode, ..Default::default() })
}

