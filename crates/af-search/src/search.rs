//! Search core built on ripgrep crates (grep, ignore, regex).

use crate::filters::SearchFilters;
use anyhow::Result;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: usize,
    pub column: usize,
    pub line: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SearchResult {
    pub matches: Vec<SearchMatch>,
}

pub fn search(
    pattern: &str,
    root: &Path,
    is_regex: bool,
    filters: &SearchFilters,
) -> Result<SearchResult> {
    let re = if is_regex {
        RegexBuilder::new(pattern).build()?
    } else {
        RegexBuilder::new(&regex::escape(pattern)).build()?
    };

    let mut builder = WalkBuilder::new(root);
    builder.hidden(!filters.hidden);
    builder.git_ignore(!filters.no_ignore);
    builder.git_global(!filters.no_ignore);
    builder.git_exclude(!filters.no_ignore);
    builder.ignore(!filters.no_ignore);

    let mut result = SearchResult::default();
    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if let Some(ext) = &filters.file_type {
            let ok = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case(ext))
                .unwrap_or(false);
            if !ok {
                continue;
            }
        }
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if bytes.contains(&0) {
            continue; // skip binary
        }
        let text = match String::from_utf8(bytes) {
            Ok(t) => t,
            Err(_) => continue,
        };
        for (i, line) in text.lines().enumerate() {
            if let Some(m) = re.find(line) {
                result.matches.push(SearchMatch {
                    path: path.to_string_lossy().to_string(),
                    line_number: i + 1,
                    column: m.start() + 1,
                    line: line.to_string(),
                });
            }
        }
    }
    Ok(result)
}
