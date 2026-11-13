//! Encoding detection and transcoding.
//!
//! Design: BOM detection first (UTF-8/UTF-16 LE/BE), then strict UTF-8 validation.
//! Ambiguous encodings must be explicit; no silent lossy conversion.
