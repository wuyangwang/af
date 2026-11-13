//! Syntax highlighting for human output (syntect).

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub fn highlight_to_ansi(content: &str, path_hint: Option<&str>) -> Option<String> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = path_hint
        .and_then(|p| std::path::Path::new(p).extension().and_then(|e| e.to_str()))
        .and_then(|ext| ps.find_syntax_by_extension(ext))
        .or_else(|| path_hint.and_then(|p| ps.find_syntax_for_file(p).ok().flatten()))
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let theme = ts.themes.get("base16-ocean.dark").or_else(|| ts.themes.values().next())?;
    let mut h = HighlightLines::new(syntax, theme);
    let mut out = String::new();
    for line in LinesWithEndings::from(content) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).ok()?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        out.push_str(&escaped);
    }
    out.push_str("\x1b[0m");
    Some(out)
}
