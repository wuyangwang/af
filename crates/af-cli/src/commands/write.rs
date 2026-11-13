use af_core::newline::NewlineStyle;
use af_core::writer::{write_with_options, EditMode, WriteOptions};
use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct WriteArgs {
    /// Target file path
    pub path: PathBuf,
    /// Read input from stdin
    #[arg(long)]
    pub stdin: bool,
    /// Read input from another file
    #[arg(long)]
    pub from: Option<PathBuf>,
    /// Explicit encoding (e.g. utf-8, gbk)
    #[arg(long)]
    pub encoding: Option<String>,
    /// Write UTF-8 BOM (true/false). If omitted, preserve or default.
    #[arg(long)]
    pub bom: Option<bool>,
    /// Line ending: lf, crlf, cr
    #[arg(long)]
    pub eol: Option<String>,
    /// Normalize output to UTF-8 no BOM + LF
    #[arg(long)]
    pub normalize: bool,
    /// Strict mode: reject if target format does not meet policy
    #[arg(long)]
    pub strict: bool,
    /// Preview only, do not write
    #[arg(long)]
    pub dry_run: bool,
    /// Allow overwriting an existing file
    #[arg(long)]
    pub force: bool,
    /// Expected SHA-256 of existing file before overwrite
    #[arg(long)]
    pub expected_hash: Option<String>,
}

fn read_input(args: &WriteArgs) -> Result<String> {
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

fn parse_eol(s: &str) -> Option<NewlineStyle> {
    match s.to_ascii_lowercase().as_str() {
        "lf" => Some(NewlineStyle::Lf),
        "crlf" => Some(NewlineStyle::CrLf),
        "cr" => Some(NewlineStyle::Cr),
        _ => None,
    }
}

pub fn run(args: WriteArgs, json_mode: bool) -> Result<()> {
    af_core::path_safety::ensure_safe_path(&args.path).map_err(|e| anyhow::anyhow!(e))?;
    if args.path.exists() && !args.force {
        return Err(anyhow::anyhow!(af_core::errors::AfError::safety(
            "TARGET_EXISTS",
            format!("target exists; pass --force to overwrite: {}", args.path.display())
        )));
    }
    let input = read_input(&args)?;
    if let Some(expected) = &args.expected_hash {
        if args.path.exists() {
            let existing = af_core::reader::read_text_file(&args.path, None).map_err(|e| anyhow::anyhow!(e))?;
            if &existing.meta.sha256 != expected {
                return Err(anyhow::anyhow!(af_core::errors::AfError::Conflict {
                    code: "HASH_CONFLICT".into(),
                    message: format!("hash conflict: expected {} but found {}", expected, existing.meta.sha256),
                    path: Some(args.path.display().to_string()),
                }));
            }
        } else {
            return Err(anyhow::anyhow!(af_core::errors::AfError::invalid_input(
                "--expected-hash provided but target file does not exist"
            )));
        }
    }
    let mode = if args.strict {
        EditMode::Strict
    } else if args.normalize {
        EditMode::Normalize
    } else {
        EditMode::Preserve
    };
    let eol = args.eol.as_deref().and_then(parse_eol);
    if args.eol.is_some() && eol.is_none() {
        bail!("invalid --eol value (expected lf|crlf|cr)");
    }
    let opts = WriteOptions {
        mode,
        encoding: args.encoding.clone(),
        bom: args.bom,
        eol,
    };

    if args.dry_run {
        let plan = af_core::writer::plan_write(&args.path, &input, &opts).map_err(|e| anyhow::anyhow!(e))?;
        if json_mode {
            let out = json!({
                "ok": true,
                "command": "write",
                "path": args.path.to_string_lossy(),
                "dryRun": true,
                "bytes": plan.meta.size,
                "encoding": plan.meta.encoding,
                "bom": plan.meta.bom,
                "eol": plan.meta.eol,
                "sha256": plan.meta.sha256
            });
            println!("{}", af_render::json::success(out));
        } else {
            println!("dry-run: would write {} bytes to {}", plan.meta.size, args.path.display());
            println!("encoding: {} bom: {} eol: {}", plan.meta.encoding, plan.meta.bom, plan.meta.eol);
            println!("sha256: {}", plan.meta.sha256);
        }
        return Ok(());
    }

    let outcome = write_with_options(&args.path, &input, &opts).map_err(|e| anyhow::anyhow!(e))?;

    if json_mode {
        let out = json!({
            "ok": true,
            "command": "write",
            "path": args.path.to_string_lossy(),
            "bytesWritten": outcome.bytes_written,
            "encoding": outcome.meta.encoding,
            "bom": outcome.meta.bom,
            "eol": outcome.meta.eol,
            "sha256": outcome.meta.sha256
        });
        println!("{}", af_render::json::success(out));
    } else {
        println!("wrote {} bytes to {}", outcome.bytes_written, args.path.display());
    }
    Ok(())
}
