//! Search core built on ripgrep crates (grep, ignore, regex).

use crate::filters::SearchFilters;
use anyhow::Result;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ContextLine {
    pub line_number: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: usize,
    pub column: usize,
    pub line: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub before: Vec<ContextLine>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub after: Vec<ContextLine>,
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
        RegexBuilder::new(pattern).case_insensitive(filters.ignore_case).build()?
    } else {
        RegexBuilder::new(&regex::escape(pattern)).case_insensitive(filters.ignore_case).build()?
    };

    let mut builder = WalkBuilder::new(root);
    builder.hidden(!filters.hidden);
    builder.git_ignore(!filters.no_ignore);
    builder.git_global(!filters.no_ignore);
    builder.git_exclude(!filters.no_ignore);
    builder.ignore(!filters.no_ignore);
    if let Some(glob) = &filters.glob {
        let mut overrides = ignore::overrides::OverrideBuilder::new(root);
        overrides.add(glob)?;
        builder.overrides(overrides.build()?);
    }

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
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if let Some(m) = re.find(line) {
                let before_start = i.saturating_sub(filters.before);
                let before = lines[before_start..i]
                    .iter()
                    .enumerate()
                    .map(|(j, t)| ContextLine { line_number: before_start + j + 1, text: (*t).to_string() })
                    .collect();
                let after_end = (i + 1 + filters.after).min(lines.len());
                let after = lines[i + 1..after_end]
                    .iter()
                    .enumerate()
                    .map(|(j, t)| ContextLine { line_number: i + 2 + j, text: (*t).to_string() })
                    .collect();
                result.matches.push(SearchMatch {
                    path: path.to_string_lossy().to_string(),
                    line_number: i + 1,
                    column: m.start() + 1,
                    line: (*line).to_string(),
                    before,
                    after,
                });
            }
        }
    }
    Ok(result)
}
