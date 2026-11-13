use af_core::encoding::encode_from_string;
use af_core::reader::read_text_file;
use af_core::writer::write_new_file;
use af_core::atomic::atomic_write;
use af_core::hash::sha256_hex;
use af_core::newline::NewlineStyle;
use af_render::json as rjson;
use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct AppendArgs {
    /// Target file path
    pub path: PathBuf,
    /// Read input from stdin
    #[arg(long)]
    pub stdin: bool,
    /// Read input from another file
    #[arg(long)]
    pub from: Option<PathBuf>,
    /// Ensure appended content ends with a newline
    #[arg(long)]
    pub ensure_newline: bool,
    /// Preview only, do not write
    #[arg(long)]
    pub dry_run: bool,
    /// Expected SHA-256 of existing file before append
    #[arg(long)]
    pub expected_hash: Option<String>,
}

fn read_input(args: &AppendArgs) -> Result<String> {
    if args.from.is_some() && args.stdin {
        bail!("--stdin and --from are mutually exclusive");
    }
    if let Some(from) = &args.from {
        return Ok(fs::read_to_string(from)?);
    }
    if args.stdin || args.from.is_none() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    bail!("no input provided");
}

pub fn run(args: AppendArgs, json_mode: bool) -> Result<()> {
    af_core::path_safety::ensure_safe_path(&args.path).map_err(|e| anyhow::anyhow!(e))?;
    let mut input = read_input(&args)?;
    if !args.path.exists() {
        if args.ensure_newline && !input.ends_with('\n') && !input.ends_with('\r') {
            input.push('\n');
        }
        let planned = af_core::writer::plan_write(&args.path, &input, &af_core::writer::WriteOptions::default())
            .map_err(|e| anyhow::anyhow!(e))?;
        if args.dry_run {
            if json_mode {
                let out = json!({"ok": true, "command": "append", "path": args.path.to_string_lossy(), "dryRun": true, "created": true, "bytes": planned.meta.size, "encoding": planned.meta.encoding, "bom": planned.meta.bom, "eol": planned.meta.eol, "sha256": planned.meta.sha256, "contentToAppend": input});
                println!("{}", rjson::success(out));
            } else {
                println!("dry-run: would create {} and append {} bytes", args.path.display(), input.len());
                println!("encoding: {} bom: {} eol: {} sha256: {}", planned.meta.encoding, planned.meta.bom, planned.meta.eol, planned.meta.sha256);
            }
            return Ok(());
        }
        let outcome = write_new_file(&args.path, &input)?;
        if json_mode {
            let out = json!({"ok": true, "command": "append", "path": args.path.to_string_lossy(), "created": true, "bytesWritten": outcome.bytes_written, "sha256": outcome.meta.sha256, "encoding": outcome.meta.encoding, "bom": outcome.meta.bom, "eol": outcome.meta.eol});
            println!("{}", rjson::success(out));
        } else {
            println!("created and appended {} bytes to {}", outcome.bytes_written, args.path.display());
        }
        return Ok(());
    }

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
    if existing.meta.mixed_newlines {
        return Err(anyhow::anyhow!(af_core::errors::AfError::Safety {
            code: "MIXED_NEWLINE".into(),
            message: "mixed newline file: refuse to append without explicit normalization strategy".into(),
            path: Some(args.path.display().to_string()),
        }));
    }
    let _eol = existing.newline.sequence();
    if args.ensure_newline {
        let ends_with_nl = input.ends_with('\n') || input.ends_with('\r');
        if !ends_with_nl {
            input.push_str(if matches!(existing.newline, NewlineStyle::CrLf) { "\r\n" } else if matches!(existing.newline, NewlineStyle::Cr) { "\r" } else { "\n" });
        }
    }
    let mut new_content = existing.content.clone();
    new_content.push_str(&input);
    let bytes = encode_from_string(&new_content, &existing.detected.label, existing.detected.bom)?;
    let sha = sha256_hex(&bytes);

    if args.dry_run {
        if json_mode {
            let out = json!({"ok": true, "command": "append", "path": args.path.to_string_lossy(), "dryRun": true, "bytesAppended": input.len(), "encoding": existing.detected.label, "bom": existing.meta.bom, "eol": existing.meta.eol, "sha256": sha, "contentToAppend": input});
            println!("{}", rjson::success(out));
        } else {
            println!("dry-run: would append {} bytes to {}", input.len(), args.path.display());
            println!("encoding: {} bom: {} eol: {} finalSha256: {}", existing.detected.label, existing.meta.bom, existing.meta.eol, sha);
        }
        return Ok(());
    }

    // basic state recheck before atomic replace
    let recheck = read_text_file(&args.path, None).map_err(|e| anyhow::anyhow!(e))?;
    if recheck.meta.sha256 != existing.meta.sha256 {
        return Err(anyhow::anyhow!(af_core::errors::AfError::Conflict {
            code: "HASH_CONFLICT".into(),
            message: "file changed during append".into(),
            path: Some(args.path.display().to_string()),
        }));
    }
    atomic_write(&args.path, &bytes)?;
    if json_mode {
        let out = json!({"ok": true, "command": "append", "path": args.path.to_string_lossy(), "bytesAppended": input.len(), "encoding": existing.detected.label, "bom": existing.meta.bom, "eol": existing.meta.eol, "sha256": sha});
        println!("{}", rjson::success(out));
    } else {
        println!("appended {} bytes to {}", input.len(), args.path.display());
    }
    Ok(())
}
