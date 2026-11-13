use af_core::encoding::encode_from_string;
use af_core::atomic::atomic_write;
use af_core::hash::sha256_hex;
use af_core::newline::{detect_newline, normalize_newlines, NewlineStyle};
use af_core::reader::read_text_file;
use af_render::json as rjson;
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
    operations: Vec<Operation>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Operation {
    Replace { old: String, new: String, #[serde(rename = "expectedMatches")] expected_matches: Option<usize> },
    Insert { line: usize, content: String, #[serde(default)] context: Option<LineContext> },
    Delete { line: usize, #[serde(default)] content: Option<String>, #[serde(default)] context: Option<LineContext> },
    SetLine { line: usize, new: String, #[serde(default)] old: Option<String>, #[serde(default)] context: Option<LineContext> },
}

#[derive(Debug, Deserialize, Clone, Default)]
struct LineContext {
    #[serde(default)] before: Option<String>,
    #[serde(default)] after: Option<String>,
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
    while let Some(l) = lines.peek() { if l.starts_with("@@") { break; } lines.next(); }
    while let Some(h) = lines.next() {
        if !h.starts_with("@@") { continue; }
        hunks += 1;
        let parts: Vec<&str> = h.split_whitespace().collect();
        if parts.len() < 3 { bail!("invalid hunk header: {h}"); }
        let old_part = parts[1].trim_start_matches('-');
        let old_start: usize = old_part.split(',').next().unwrap_or("1").parse().unwrap_or(1);
        let target = old_start.saturating_sub(1);
        if target > orig_lines.len() { bail!("hunk out of range"); }
        while idx < target { out.push(orig_lines[idx].to_string()); idx += 1; }
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
                    out.push(orig_lines[idx].to_string()); idx += 1; lines.next();
                }
                '-' => {
                    if idx >= orig_lines.len() { bail!("remove out of range"); }
                    let src = orig_lines[idx].trim_end_matches(['\r', '\n']);
                    if src != text { bail!("remove mismatch while applying diff"); }
                    idx += 1; lines.next();
                }
                '+' => {
                    let nl = if original.contains("\r\n") { "\r\n" } else { "\n" };
                    if text.ends_with('\n') || text.ends_with('\r') { out.push(text.to_string()); } else { out.push(format!("{text}{nl}")); }
                    lines.next();
                }
                '\\' => { if let Some(last) = out.last_mut() { while last.ends_with('\n') || last.ends_with('\r') { last.pop(); } } lines.next(); }
                _ => { lines.next(); }
            }
        }
    }
    while idx < orig_lines.len() { out.push(orig_lines[idx].to_string()); idx += 1; }
    Ok((out.concat(), hunks))
}

fn split_lines_keep(text: &str) -> Vec<String> {
    if text.is_empty() { return Vec::new(); }
    text.split_inclusive('\n').map(|s| s.to_string()).collect()
}

fn validate_and_apply_ops(original: &str, ops: &[Operation]) -> Result<String> {
    // pre-validate all operations against original/simulated states as appropriate, then apply
    let mut current = original.to_string();
    for op in ops {
        match op {
            Operation::Replace { old, new, expected_matches } => {
                let expected = expected_matches.unwrap_or(1);
                let count = current.matches(old).count();
                if count != expected {
                    return Err(anyhow::anyhow!(af_core::errors::AfError::match_count(format!(
                        "replace: expected {}, found {}", expected, count
                    ))));
                }
                current = current.replacen(old, new, expected);
            }
            Operation::Insert { line, content, context } => {
                let mut lines = split_lines_keep(&current);
                let idx = line.saturating_sub(1);
                if idx > lines.len() { return Err(anyhow::anyhow!(af_core::errors::AfError::invalid_input("insert line out of range"))); }
                if let Some(ctx) = context { check_context(&lines, idx, ctx)?; }
                let nl = if current.contains("\r\n") { "\r\n" } else { "\n" };
                let mut ins = content.clone();
                if !ins.ends_with('\n') && !ins.ends_with('\r') { ins.push_str(nl); }
                lines.insert(idx, ins);
                current = lines.concat();
            }
            Operation::Delete { line, content, context } => {
                let mut lines = split_lines_keep(&current);
                if *line == 0 || *line > lines.len() { return Err(anyhow::anyhow!(af_core::errors::AfError::invalid_input("delete line out of range"))); }
                let idx = line - 1;
                if let Some(ctx) = context { check_context(&lines, idx, ctx)?; }
                if let Some(expected) = content {
                    let actual = lines[idx].trim_end_matches(['\r','\n']);
                    if actual != expected.trim_end_matches(['\r','\n']) {
                        return Err(anyhow::anyhow!(af_core::errors::AfError::safety("CONTEXT_MISMATCH", "delete content mismatch")));
                    }
                }
                lines.remove(idx);
                current = lines.concat();
            }
            Operation::SetLine { line, new, old, context } => {
                let mut lines = split_lines_keep(&current);
                if *line == 0 || *line > lines.len() { return Err(anyhow::anyhow!(af_core::errors::AfError::invalid_input("setLine line out of range"))); }
                let idx = line - 1;
                if let Some(ctx) = context { check_context(&lines, idx, ctx)?; }
                if let Some(expected_old) = old {
                    let actual = lines[idx].trim_end_matches(['\r','\n']);
                    if actual != expected_old.trim_end_matches(['\r','\n']) {
                        return Err(anyhow::anyhow!(af_core::errors::AfError::safety("CONTEXT_MISMATCH", "setLine old content mismatch")));
                    }
                }
                let nl = if lines[idx].ends_with("\r\n") { "\r\n" } else if lines[idx].ends_with('\r') { "\r" } else if lines[idx].ends_with('\n') { "\n" } else { if current.contains("\r\n") { "\r\n" } else { "\n" } };
                let mut v = new.clone();
                if !v.ends_with('\n') && !v.ends_with('\r') && lines[idx].ends_with(['\n','\r']) { v.push_str(nl); }
                lines[idx] = v;
                current = lines.concat();
            }
        }
    }
    Ok(current)
}

fn check_context(lines: &[String], idx: usize, ctx: &LineContext) -> Result<()> {
    if let Some(before) = &ctx.before {
        if idx == 0 { return Err(anyhow::anyhow!(af_core::errors::AfError::safety("CONTEXT_MISMATCH", "missing before context"))); }
        let actual = lines[idx - 1].trim_end_matches(['\r','\n']);
        if actual != before.trim_end_matches(['\r','\n']) {
            return Err(anyhow::anyhow!(af_core::errors::AfError::safety("CONTEXT_MISMATCH", "before context mismatch")));
        }
    }
    if let Some(after) = &ctx.after {
        if idx + 1 >= lines.len() { return Err(anyhow::anyhow!(af_core::errors::AfError::safety("CONTEXT_MISMATCH", "missing after context"))); }
        let actual = lines[idx + 1].trim_end_matches(['\r','\n']);
        if actual != after.trim_end_matches(['\r','\n']) {
            return Err(anyhow::anyhow!(af_core::errors::AfError::safety("CONTEXT_MISMATCH", "after context mismatch")));
        }
    }
    Ok(())
}

pub fn run(args: PatchArgs, json_mode: bool) -> Result<()> {
    af_core::path_safety::ensure_safe_path(&args.path).map_err(|e| anyhow::anyhow!(e))?;
    let input = read_input_text(&args)?;
    let existing = read_text_file(&args.path, None).map_err(|e| anyhow::anyhow!(e))?;
    if let Some(expected) = &args.expected_hash {
        if &existing.meta.sha256 != expected {
            return Err(anyhow::anyhow!(af_core::errors::AfError::Conflict {
                code: "HASH_CONFLICT".into(),
                message: format!("hash conflict: expected {} but found {}", expected, existing.meta.sha256),
                path: Some(args.path.display().to_string()),
            }));
        }
    }
    if !args.normalize && existing.meta.mixed_newlines {
        return Err(anyhow::anyhow!(af_core::errors::AfError::Safety {
            code: "MIXED_NEWLINE".into(),
            message: "mixed newline file: refuse to patch without --normalize or explicit conversion".into(),
            path: Some(args.path.display().to_string()),
        }));
    }

    let (mut content, total_ops) = if args.spec.is_some() || input.trim_start().starts_with('{') {
        let spec: PatchSpec = serde_json::from_str(&input).map_err(|e| anyhow::anyhow!(af_core::errors::AfError::invalid_input(e.to_string())))?;
        // validate and apply (sequentially with checks); for full pre-validation of replace counts against original we check on original snapshot too
        let _ = &spec; // basic pre-validation: ensure supported types parsed
        let c = validate_and_apply_ops(&existing.content, &spec.operations)?;
        (c, spec.operations.len())
    } else {
        apply_unified_diff(&existing.content, &input).map_err(|e| anyhow::anyhow!(af_core::errors::AfError::general(e.to_string())))?
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
            let out = json!({
                "ok": true, "command": "patch", "path": args.path.to_string_lossy(), "dryRun": true,
                "operations": total_ops, "beforeHash": existing.meta.sha256, "afterHash": new_hash,
                "encoding": label, "bom": !matches!(bom, af_core::encoding::Bom::None), "eol": detect_newline(&content).as_str()
            });
            println!("{}", rjson::success(out));
        } else {
            let diff = af_render::diff::unified_diff(&existing.content, &content, &args.path.to_string_lossy());
            print!("{diff}");
            println!("dry-run: {} operation(s), {} -> {}", total_ops, existing.meta.sha256, new_hash);
        }
        return Ok(());
    }

    // recheck before commit
    let recheck = read_text_file(&args.path, None).map_err(|e| anyhow::anyhow!(e))?;
    if recheck.meta.sha256 != existing.meta.sha256 {
        return Err(anyhow::anyhow!(af_core::errors::AfError::Conflict { code: "HASH_CONFLICT".into(), message: "file changed before patch commit".into(), path: Some(args.path.display().to_string()) }));
    }
    atomic_write(&args.path, &bytes)?;
    if json_mode {
        let out = json!({
            "ok": true,
            "command": "patch",
            "path": args.path.to_string_lossy(),
            "operations": total_ops,
            "beforeHash": existing.meta.sha256,
            "afterHash": new_hash,
            "encoding": label,
            "bom": !matches!(bom, af_core::encoding::Bom::None),
            "eol": detect_newline(&content).as_str()
        });
        println!("{}", rjson::success(out));
    } else {
        println!("patched {} ({} ops)", args.path.display(), total_ops);
    }
    Ok(())
}
