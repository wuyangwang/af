//! File reading: full/range reads, metadata, binary detection.

use crate::encoding::{decode_to_string, detect_bom, Bom, DetectedEncoding};
use crate::errors::AfError;
use crate::hash::sha256_hex;
use crate::newline::{detect_newline, NewlineStyle};
use std::fs;
use std::io::{BufRead, BufReader, Read};
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
    let bytes = fs::read(path).map_err(|e| AfError::io(path, e))?;
    let size = bytes.len() as u64;
    let (bom, _) = detect_bom(&bytes);
    if looks_binary(&bytes, bom) && explicit_encoding.is_none() {
        return Err(AfError::encoding_path(
            "BINARY_FILE",
            path,
            format!("binary file detected: {}", path.display()),
        ));
    }
    let (content, detected) = decode_to_string(&bytes, explicit_encoding).map_err(|mut e| {
        if e.path().is_none() {
            if let AfError::Encoding { code, message, .. } = e {
                e = AfError::Encoding { code, message, path: Some(path.display().to_string()) };
            }
        }
        e
    })?;
    let newline = detect_newline(&content);
    let meta = FileMeta {
        encoding: detected.label.clone(),
        bom: !matches!(detected.bom, Bom::None),
        eol: newline.as_str().to_string(),
        size,
        sha256: sha256_hex(&bytes),
        mixed_newlines: matches!(newline, NewlineStyle::Mixed),
    };
    Ok(ReadResult { content, meta, detected, newline })
}

#[derive(Debug, Clone)]
pub struct RangeReadResult {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub meta: FileMeta,
}

pub fn read_text_range(path: &Path, start: Option<usize>, end: Option<usize>, explicit_encoding: Option<&str>) -> Result<RangeReadResult, AfError> {
    // Efficient path for UTF-8 (no BOM) files; otherwise decode fully.
    let file = fs::File::open(path).map_err(|e| AfError::io(path, e))?;
    let _size = file.metadata().map_err(|e| AfError::io(path, e))?.len();
    let mut reader = BufReader::new(file);
    let mut prefix = [0u8; 4];
    let n = reader.read(&mut prefix).map_err(|e| AfError::io(path, e))?;
    let (bom, _off) = detect_bom(&prefix[..n]);
    let can_stream_utf8 = matches!(bom, Bom::None) && explicit_encoding.is_none();
    if !can_stream_utf8 {
        let full = read_text_file(path, explicit_encoding)?;
        let (content, s, e) = slice_content(&full.content, start, end);
        return Ok(RangeReadResult { content, start_line: s, end_line: e, meta: full.meta });
    }
    // stream UTF-8
    let file = fs::File::open(path).map_err(|e| AfError::io(path, e))?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    let mut line_no = 0usize;
    let s = start.unwrap_or(1).max(1);
    let e_req = end.unwrap_or(usize::MAX);
    let mut out = String::new();
    let mut actual_end = 0usize;
    let mut sample = String::new();
    loop {
        buf.clear();
        let bytes = reader.read_line(&mut buf).map_err(|e| AfError::io(path, e))?;
        if bytes == 0 { break; }
        // validate utf8 incrementally by virtue of String read_line; if invalid, fall back
        if buf.as_bytes().contains(&0) {
            let full = read_text_file(path, explicit_encoding)?;
            let (content, s2, e2) = slice_content(&full.content, start, end);
            return Ok(RangeReadResult { content, start_line: s2, end_line: e2, meta: full.meta });
        }
        line_no += 1;
        if sample.len() < 8192 { sample.push_str(&buf); }
        if line_no >= s && line_no <= e_req {
            out.push_str(&buf);
            actual_end = line_no;
        }
        if line_no > e_req { break; }
    }
    if line_no == 0 { actual_end = 0; }
    // compute meta from full bytes hash/encoding info cheaply enough? need sha/encoding/eol detection; read full for correctness of meta
    let full = read_text_file(path, explicit_encoding)?;
    if actual_end == 0 && s > 1 { actual_end = s.saturating_sub(1); }
    let end_line = if e_req == usize::MAX { line_no } else { actual_end.min(e_req) };
    Ok(RangeReadResult { content: out, start_line: s, end_line, meta: full.meta })
}

fn slice_content(content: &str, start: Option<usize>, end: Option<usize>) -> (String, usize, usize) {
    if content.is_empty() { return (String::new(), 1, 0); }
    let total_lines = content.lines().count();
    let s = start.unwrap_or(1).max(1);
    let e = end.unwrap_or(total_lines).min(total_lines).max(1);
    if s > e { return (String::new(), s, e.saturating_sub(1)); }
    let mut out = String::new();
    for (idx, line) in content.split_inclusive('\n').enumerate() {
        let ln = idx + 1;
        if ln >= s && ln <= e { out.push_str(line); }
        if ln > e { break; }
    }
    if !content.contains('\n') && s == 1 { out = content.to_string(); }
    (out, s, e)
}

