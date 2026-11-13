//! Newline detection and normalization (LF/CRLF/CR/mixed).
//!
//! Preserve mode keeps the original style; mixed newlines require explicit handling.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineStyle {
    Lf,
    CrLf,
    Cr,
    Mixed,
    None,
}

impl NewlineStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            NewlineStyle::Lf => "lf",
            NewlineStyle::CrLf => "crlf",
            NewlineStyle::Cr => "cr",
            NewlineStyle::Mixed => "mixed",
            NewlineStyle::None => "none",
        }
    }

    pub fn sequence(&self) -> &'static str {
        match self {
            NewlineStyle::Lf | NewlineStyle::None | NewlineStyle::Mixed => "\n",
            NewlineStyle::CrLf => "\r\n",
            NewlineStyle::Cr => "\r",
        }
    }
}

pub fn detect_newline(text: &str) -> NewlineStyle {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut has_lf = false;
    let mut has_crlf = false;
    let mut has_cr = false;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    has_crlf = true;
                    i += 2;
                } else {
                    has_cr = true;
                    i += 1;
                }
            }
            b'\n' => {
                has_lf = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    let kinds = [has_lf, has_crlf, has_cr].iter().filter(|&&v| v).count();
    match kinds {
        0 => NewlineStyle::None,
        1 if has_lf => NewlineStyle::Lf,
        1 if has_crlf => NewlineStyle::CrLf,
        1 if has_cr => NewlineStyle::Cr,
        _ => NewlineStyle::Mixed,
    }
}

pub fn normalize_newlines(text: &str, style: NewlineStyle) -> String {
    // Normalize all line endings to \n first, then convert.
    let mut normalized = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    i += 2;
                } else {
                    i += 1;
                }
                normalized.push('\n');
            }
            b'\n' => {
                normalized.push('\n');
                i += 1;
            }
            _ => {
                let ch = text[i..].chars().next().unwrap();
                normalized.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    match style {
        NewlineStyle::Lf | NewlineStyle::None | NewlineStyle::Mixed => normalized,
        NewlineStyle::CrLf => normalized.replace('\n', "\r\n"),
        NewlineStyle::Cr => normalized.replace('\n', "\r"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_lf_crlf_cr_mixed() {
        assert_eq!(detect_newline("a\nb\n"), NewlineStyle::Lf);
        assert_eq!(detect_newline("a\r\nb\r\n"), NewlineStyle::CrLf);
        assert_eq!(detect_newline("a\rb\r"), NewlineStyle::Cr);
        assert_eq!(detect_newline("a\nb\r\n"), NewlineStyle::Mixed);
        assert_eq!(detect_newline("abc"), NewlineStyle::None);
    }

    #[test]
    fn normalizes_to_lf_and_crlf() {
        assert_eq!(normalize_newlines("a\r\nb\r", NewlineStyle::Lf), "a\nb\n");
        assert_eq!(normalize_newlines("a\nb\n", NewlineStyle::CrLf), "a\r\nb\r\n");
    }
}

