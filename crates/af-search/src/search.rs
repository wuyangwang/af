//! Search core built on ripgrep crates (grep-regex, grep-searcher, ignore).

use crate::filters::SearchFilters;
use anyhow::Result;
use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::SearcherBuilder;
use ignore::WalkBuilder;
use serde::Serialize;
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
    let mut mb = RegexMatcherBuilder::new();
    mb.case_insensitive(filters.ignore_case);
    let matcher = if is_regex {
        mb.build(pattern)?
    } else {
        mb.fixed_strings(true).build(pattern)?
    };

    let mut builder = WalkBuilder::new(root);
    builder.hidden(!filters.hidden);
    builder.git_ignore(!filters.no_ignore);
    builder.git_global(!filters.no_ignore);
    builder.git_exclude(!filters.no_ignore);
    builder.ignore(!filters.no_ignore);
    builder.follow_links(false);
    if let Some(glob) = &filters.glob {
        let mut overrides = ignore::overrides::OverrideBuilder::new(root);
        overrides.add(glob)?;
        builder.overrides(overrides.build()?);
    }

    let mut searcher = SearcherBuilder::new()
        .line_number(true)
        .before_context(filters.before)
        .after_context(filters.after)
        .multi_line(false)
        .build();

    let mut result = SearchResult::default();
    for entry in builder.build() {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if let Some(ext) = &filters.file_type {
            let ok = path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case(ext)).unwrap_or(false);
            if !ok { continue; }
        }
        let path_str = path.to_string_lossy().to_string();
        let mut pending: Option<SearchMatch> = None;
        let mut after_remaining = 0usize;
        let _ = searcher.search_path(&matcher, path, UTF8(|lnum, line| {
            let text = line.trim_end_matches(['\r', '\n']).to_string();
            // grep-searcher reports context/matches; we detect matches via matcher on the line content
            let is_match = matcher.is_match(line.as_bytes()).unwrap_or(false);
            if is_match {
                if let Some(p) = pending.take() { result.matches.push(p); }
                // find all match columns in this line
                let mut columns = Vec::new();
                let _ = matcher.find_iter(line.as_bytes(), |m| { columns.push(m.start() + 1); true });
                if columns.is_empty() { columns.push(1); }
                let before_vec = Vec::new(); // simplified: before context collected not available per-callback easily; keep basic
                for col in columns {
                    let m = SearchMatch {
                        path: path_str.clone(),
                        line_number: lnum as usize,
                        column: col,
                        line: text.clone(),
                        before: before_vec.clone(),
                        after: Vec::new(),
                    };
                    result.matches.push(m);
                }
                after_remaining = filters.after;
            } else if after_remaining > 0 {
                if let Some(last) = result.matches.last_mut() {
                    if last.path == path_str {
                        last.after.push(ContextLine { line_number: lnum as usize, text });
                    }
                }
                after_remaining -= 1;
            } else {
                // before-context lines arrive here in some configurations; not reliably distinguishable in this sink.
                // Keep simple: rely on matcher results; context handled best-effort below.
                let _ = lnum;
            }
            Ok(true)
        }));
        if let Some(p) = pending.take() { result.matches.push(p); }
    }

    // Best-effort: fill before/after context by re-reading matched files (simple and correct enough)
    if filters.before > 0 || filters.after > 0 {
        use std::collections::BTreeMap;
        let mut by_file: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, m) in result.matches.iter().enumerate() { by_file.entry(m.path.clone()).or_default().push(i); }
        for (path, idxs) in by_file {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let lines: Vec<&str> = content.lines().collect();
                for idx in idxs {
                    let ln = result.matches[idx].line_number;
                    let bstart = ln.saturating_sub(1).saturating_sub(filters.before);
                    let bend = ln.saturating_sub(1);
                    result.matches[idx].before = lines[bstart..bend].iter().enumerate().map(|(j,t)| ContextLine{line_number:bstart+j+1,text:(*t).to_string()}).collect();
                    let astart = ln;
                    let aend = (ln + filters.after).min(lines.len());
                    result.matches[idx].after = lines[astart..aend].iter().enumerate().map(|(j,t)| ContextLine{line_number:astart+j+1,text:(*t).to_string()}).collect();
                }
            }
        }
    }

    Ok(result)
}
