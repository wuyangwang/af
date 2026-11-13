//! Encoding detection and transcoding.
//!
//! Design: BOM detection first (UTF-8/UTF-16 LE/BE), then strict UTF-8 validation.
//! Ambiguous encodings must be explicit; no silent lossy conversion.

use crate::errors::AfError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bom {
    None,
    Utf8,
    Utf16Le,
    Utf16Be,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedEncoding {
    pub label: String,
    pub bom: Bom,
    pub is_utf8: bool,
}

pub fn detect_bom(bytes: &[u8]) -> (Bom, usize) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        (Bom::Utf8, 3)
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        (Bom::Utf16Le, 2)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        (Bom::Utf16Be, 2)
    } else {
        (Bom::None, 0)
    }
}

pub fn decode_to_string(bytes: &[u8], explicit: Option<&str>) -> Result<(String, DetectedEncoding), AfError> {
    if bytes.contains(&0) && explicit.is_none() {
        // Heuristic: NUL often indicates binary; UTF-16 contains NULs but handled by BOM.
        // Let BOM path try first.
    }

    let (bom, offset) = detect_bom(bytes);
    match bom {
        Bom::Utf8 => {
            let text = std::str::from_utf8(&bytes[offset..])
                .map_err(|e| AfError::encoding("INVALID_ENCODING", format!("invalid UTF-8 with BOM: {e}")))?;
            Ok((
                text.to_string(),
                DetectedEncoding {
                    label: "utf-8".to_string(),
                    bom,
                    is_utf8: true,
                },
            ))
        }
        Bom::Utf16Le => {
            let (cow, _, had_errors) = encoding_rs::UTF_16LE.decode(&bytes[offset..]);
            if had_errors {
                return Err(AfError::encoding("INVALID_ENCODING", "invalid UTF-16LE content"));
            }
            Ok((
                cow.into_owned(),
                DetectedEncoding {
                    label: "utf-16le".to_string(),
                    bom,
                    is_utf8: false,
                },
            ))
        }
        Bom::Utf16Be => {
            let (cow, _, had_errors) = encoding_rs::UTF_16BE.decode(&bytes[offset..]);
            if had_errors {
                return Err(AfError::encoding("INVALID_ENCODING", "invalid UTF-16BE content"));
            }
            Ok((
                cow.into_owned(),
                DetectedEncoding {
                    label: "utf-16be".to_string(),
                    bom,
                    is_utf8: false,
                },
            ))
        }
        Bom::None => {
            if let Some(label) = explicit {
                let enc = encoding_rs::Encoding::for_label(label.as_bytes())
                    .ok_or_else(|| AfError::invalid_input(format!("unknown encoding: {label}")))?;
                let (cow, _, had_errors) = enc.decode(bytes);
                if had_errors {
                    return Err(AfError::encoding(
                        "DECODE_ERROR",
                        format!("failed to decode as {label} without loss"),
                    ));
                }
                let is_utf8 = enc == encoding_rs::UTF_8;
                return Ok((
                    cow.into_owned(),
                    DetectedEncoding {
                        label: enc.name().to_ascii_lowercase(),
                        bom: Bom::None,
                        is_utf8,
                    },
                ));
            }
            match std::str::from_utf8(bytes) {
                Ok(text) => Ok((
                    text.to_string(),
                    DetectedEncoding {
                        label: "utf-8".to_string(),
                        bom: Bom::None,
                        is_utf8: true,
                    },
                )),
                Err(_) => Err(AfError::encoding(
                    "AMBIGUOUS_ENCODING",
                    "无法可靠判断文件编码，请显式指定 --encoding",
                )),
            }
        }
    }
}

pub fn encode_from_string(
    text: &str,
    label: &str,
    bom: Bom,
) -> Result<Vec<u8>, AfError> {
    match label.to_ascii_lowercase().as_str() {
        "utf-8" | "utf8" => {
            let mut out = Vec::with_capacity(text.len() + 3);
            if bom == Bom::Utf8 {
                out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            }
            out.extend_from_slice(text.as_bytes());
            Ok(out)
        }
        "utf-16le" | "utf16le" => {
            let mut out = Vec::new();
            if bom != Bom::None {
                out.extend_from_slice(&[0xFF, 0xFE]);
            }
            for u in text.encode_utf16() {
                out.extend_from_slice(&u.to_le_bytes());
            }
            Ok(out)
        }
        "utf-16be" | "utf16be" => {
            let mut out = Vec::new();
            if bom != Bom::None {
                out.extend_from_slice(&[0xFE, 0xFF]);
            }
            for u in text.encode_utf16() {
                out.extend_from_slice(&u.to_be_bytes());
            }
            Ok(out)
        }
        other => {
            let enc = encoding_rs::Encoding::for_label(other.as_bytes())
                .ok_or_else(|| AfError::invalid_input(format!("unknown encoding: {other}")))?;
            let (cow, _, unmappable) = enc.encode(text);
            if unmappable {
                return Err(AfError::encoding(
                    "ENCODE_ERROR",
                    format!("cannot losslessly encode text as {other}"),
                ));
            }
            Ok(cow.into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_utf8_bom() {
        let (bom, off) = detect_bom(&[0xEF, 0xBB, 0xBF, b'a']);
        assert_eq!(bom, Bom::Utf8);
        assert_eq!(off, 3);
    }

    #[test]
    fn decodes_utf8_no_bom() {
        let (s, det) = decode_to_string(b"hello", None).unwrap();
        assert_eq!(s, "hello");
        assert_eq!(det.label, "utf-8");
        assert_eq!(det.bom, Bom::None);
    }

    #[test]
    fn rejects_ambiguous_without_explicit() {
        let bytes = vec![0xC4, 0xE3, 0xBA, 0xC3]; // "你好" in GBK
        assert!(decode_to_string(&bytes, None).is_err());
    }

    #[test]
    fn encodes_utf8_with_bom() {
        let bytes = encode_from_string("hi", "utf-8", Bom::Utf8).unwrap();
        assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
    }
}

