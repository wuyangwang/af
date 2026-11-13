//! Human-readable diff rendering (similar).

use similar::TextDiff;

pub fn unified_diff(old: &str, new: &str, path: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    diff.unified_diff().header(&format!("a/{path}"), &format!("b/{path}")).to_string()
}
