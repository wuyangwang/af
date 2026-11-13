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

fn read_spec(args: &PatchArgs) -> Result<PatchSpec> {
    let data = if let Some(spec) = &args.spec {
        fs::read_to_string(spec)?
    } else if args.stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        bail!("provide --spec <file> or --stdin");
    };
    Ok(serde_json::from_str(&data)?)
}

pub fn run(args: PatchArgs, json_mode: bool) -> Result<()> {
    let spec = read_spec(&args)?;
    let existing = read_text_file(&args.path, None)?;
    if let Some(expected) = &args.expected_hash {
        if &existing.meta.sha256 != expected {
            bail!("hash conflict: expected {} but found {}", expected, existing.meta.sha256);
        }
    }

    let mut content = existing.content.clone();
    let mut total_ops = 0usize;
    for op in &spec.operations {
        if op.op_type != "replace" {
            bail!("unsupported operation type: {}", op.op_type);
        }
        let count = content.matches(&op.old).count();
        let expected = op.expected_matches.unwrap_or(1);
        if count != expected {
            bail!("match count mismatch: expected {}, found {}", expected, count);
        }
        content = content.replace(&op.old, &op.new);
        total_ops += 1;
    }

    let (label, bom, eol_style) = if args.normalize {
        ("utf-8".to_string(), af_core::encoding::Bom::None, NewlineStyle::Lf)
    } else {
        (existing.detected.label.clone(), existing.detected.bom, existing.newline)
    };
    let content = normalize_newlines(&content, eol_style);
    let bytes = encode_from_string(&content, &label, bom)?;
    let new_hash = sha256_hex(&bytes);

    if args.dry_run {
        if json_mode {
            println!("{}", json!({
                "ok": true, "command": "patch", "path": args.path, "dryRun": true,
                "operations": total_ops, "beforeHash": existing.meta.sha256, "afterHash": new_hash
            }));
        } else {
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
            "bom": existing.meta.bom || args.normalize && false,
            "eol": detect_newline(&content).as_str()
        }));
    } else {
        println!("patched {} ({} ops)", args.path.display(), total_ops);
    }
    Ok(())
}
