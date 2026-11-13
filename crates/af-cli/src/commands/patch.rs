use af_core::encoding::encode_from_string;
use af_core::atomic::atomic_write;
use af_core::hash::sha256_hex;
use af_core::newline::{detect_newline, normalize_newlines, NewlineStyle};
use af_core::reader::read_text_file;
use anyhow::{bail, Result};
use clap::Args;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct PatchArgs {
    /// Target file to patch
    pub path: PathBuf,
    /// Path to JSON patch spec
    #[arg(long)]
    pub spec: Option<PathBuf>,
    /// Read spec/diff from stdin
    #[arg(long)]
    pub stdin: bool,
    /// Preview only, do not write
    #[arg(long)]
    pub dry_run: bool,
    /// Normalize output to UTF-8 no BOM + LF
    #[arg(long)]
    pub normalize: bool,
    /// Expected SHA-256 of file before patching
    #[arg(long)]
    pub expected_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatchSpec {
    #[allow(dead_code)]
    version: Option<u32>,
    operations: Vec<ReplaceOp>,
}

#[derive(Debug, Deserialize)]
struct ReplaceOp {
    #[serde(rename = "type")]
    op_type: String,
    old: String,
    new: String,
    #[serde(rename = "expectedMatches")]
    expected_matches: Option<usize>,
}

fn read_input_text(args: &PatchArgs) -> Result<String> {
    if let Some(spec) = &args.spec {
        Ok(fs::read_to_string(spec)?)
    } else if args.stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        bail!("provide --spec <file> or --stdin");
    }
}

fn apply_unified_diff(original: &str, diff: &str) -> Result<(String, usize)> {
    let orig_lines: Vec<&str> = original.split_inclusive('\n').collect();
    let mut out: Vec<String> = Vec::new();
    let mut idx = 0usize;
    let mut hunks = 0usize;

    let mut lines = diff.lines().peekable();
    // skip headers
    while let Some(l) = lines.peek() {
        if l.starts_with("@@") { break; }
        lines.next();
    }

    while let Some(h) = lines.next() {
        if !h.starts_with("@@") { continue; }
        hunks += 1;
        // parse @@ -a,b +c,d @@
        let parts: Vec<&str> = h.split_whitespace().collect();
        if parts.len() < 3 { bail!("invalid hunk header: {h}"); }
        let old_part = parts[1].trim_start_matches('-');
        let old_start: usize = old_part.split(',').next().unwrap_or("1").parse().unwrap_or(1);
        let target = old_start.saturating_sub(1);
        if target > orig_lines.len() { bail!("hunk out of range"); }
        // copy unchanged lines before hunk
        while idx < target {
            out.push(orig_lines[idx].to_string());
            idx += 1;
        }
        // apply hunk lines
        while let Some(&l) = lines.peek() {
            if l.starts_with("@@") { break; }
            if l.starts_with("---") || l.starts_with("+++") { lines.next(); continue; }
            let ch = l.chars().next().unwrap_or(' ');
            let text = if l.len() >= 1 { &l[1..] } else { "" };
            match ch {
                ' ' => {
                    if idx >= orig_lines.len() { bail!("context out of range"); }
                    let src = orig_lines[idx].trim_end_matches(['\r', '\n']);
                    if src != text { bail!("context mismatch while applying diff"); }
                    out.push(orig_lines[idx].to_string());
                    idx += 1;
                    lines.next();
                }
                '-' => {
                    if idx >= orig_lines.len() { bail!("remove out of range"); }
                    let src = orig_lines[idx].trim_end_matches(['\r', '\n']);
                    if src != text { bail!("remove mismatch while applying diff"); }
                    idx += 1;
                    lines.next();
                }
                '+' => {
                    // try to preserve newline style of original; default \n
                    let nl = if original.contains("\r\n") { "\r\n" } else { "\n" };
                    if text.ends_with('\n') || text.ends_with('\r') {
                        out.push(text.to_string());
                    } else {
                        out.push(format!("{text}{nl}"));
                    }
                    lines.next();
                }
                '\\' => { // "\ No newline at end of file"
                    if let Some(last) = out.last_mut() {
                        while last.ends_with('\n') || last.ends_with('\r') { last.pop(); }
                    }
                    lines.next();
                }
                _ => { lines.next(); }
            }
        }
    }
    // copy remainder
    while idx < orig_lines.len() {
        out.push(orig_lines[idx].to_string());
        idx += 1;
    }
    Ok((out.concat(), hunks))
}

pub fn run(args: PatchArgs, json_mode: bool) -> Result<()> {
    let input = read_input_text(&args)?;
    let existing = read_text_file(&args.path, None).map_err(|e| anyhow::anyhow!(e))?;
    if let Some(expected) = &args.expected_hash {
        if &existing.meta.sha256 != expected {
            return Err(anyhow::anyhow!(af_core::errors::AfError::Conflict(format!(
                "hash conflict: expected {} but found {}",
                expected, existing.meta.sha256
            ))));
        }
    }

    let (mut content, total_ops) = if args.spec.is_some() || input.trim_start().starts_with('{') {
        let spec: PatchSpec = serde_json::from_str(&input).map_err(|e| anyhow::anyhow!(af_core::errors::AfError::InvalidInput(e.to_string())))?;
        let mut c = existing.content.clone();
        let mut ops = 0usize;
        for op in &spec.operations {
            if op.op_type != "replace" {
                return Err(anyhow::anyhow!(af_core::errors::AfError::InvalidInput(format!(
                    "unsupported operation type: {}", op.op_type
                ))));
            }
            let count = c.matches(&op.old).count();
            let expected = op.expected_matches.unwrap_or(1);
            if count != expected {
                return Err(anyhow::anyhow!(af_core::errors::AfError::MatchCount(format!(
                    "expected {}, found {}", expected, count
                ))));
            }
            c = c.replace(&op.old, &op.new);
            ops += 1;
        }
        (c, ops)
    } else {
        apply_unified_diff(&existing.content, &input).map_err(|e| anyhow::anyhow!(af_core::errors::AfError::General(e.to_string())))?
    };

    let (label, bom, eol_style) = if args.normalize {
        ("utf-8".to_string(), af_core::encoding::Bom::None, NewlineStyle::Lf)
    } else {
        (existing.detected.label.clone(), existing.detected.bom, existing.newline)
    };
    content = normalize_newlines(&content, eol_style);
    let bytes = encode_from_string(&content, &label, bom)?;
    let new_hash = sha256_hex(&bytes);

    if args.dry_run {
        if json_mode {
            println!("{}", json!({
                "ok": true, "command": "patch", "path": args.path, "dryRun": true,
                "operations": total_ops, "beforeHash": existing.meta.sha256, "afterHash": new_hash
            }));
        } else {
            let diff = af_render::diff::unified_diff(&existing.content, &content, &args.path.to_string_lossy());
            print!("{diff}");
            println!("dry-run: {} operation(s), {} -> {}", total_ops, existing.meta.sha256, new_hash);
        }
        return Ok(());
    }

    atomic_write(&args.path, &bytes)?;
    if json_mode {
        println!("{}", json!({
            "ok": true,
            "command": "patch",
            "path": args.path,
            "operations": total_ops,
            "beforeHash": existing.meta.sha256,
            "afterHash": new_hash,
            "encoding": label,
            "bom": !matches!(bom, af_core::encoding::Bom::None),
            "eol": detect_newline(&content).as_str()
        }));
    } else {
        println!("patched {} ({} ops)", args.path.display(), total_ops);
    }
    Ok(())
}
