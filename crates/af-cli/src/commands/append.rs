use af_core::encoding::encode_from_string;
use af_core::reader::read_text_file;
use af_core::writer::write_new_file;
use af_core::atomic::atomic_write;
use af_core::hash::sha256_hex;
use af_core::newline::NewlineStyle;
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
    let mut input = read_input(&args)?;
    if !args.path.exists() {
        if args.ensure_newline && !input.ends_with('\n') && !input.ends_with('\r') {
            input.push('\n');
        }
        if args.dry_run {
            if json_mode {
                println!("{}", json!({"ok": true, "command": "append", "path": args.path, "dryRun": true, "created": true}));
            } else {
                println!("dry-run: would create and append to {}", args.path.display());
            }
            return Ok(());
        }
        let outcome = write_new_file(&args.path, &input)?;
        if json_mode {
            println!("{}", json!({"ok": true, "command": "append", "path": args.path, "created": true, "bytesWritten": outcome.bytes_written, "sha256": outcome.meta.sha256}));
        } else {
            println!("created and appended {} bytes to {}", outcome.bytes_written, args.path.display());
        }
        return Ok(());
    }

    let existing = read_text_file(&args.path, None)?;
    let eol = existing.newline.sequence();
    if args.ensure_newline {
        let ends_with_nl = input.ends_with('\n') || input.ends_with('\r');
        if !ends_with_nl {
            input.push_str(if matches!(existing.newline, NewlineStyle::None | NewlineStyle::Mixed | NewlineStyle::Lf) { "\n" } else { eol });
        }
    }
    let mut new_content = existing.content.clone();
    new_content.push_str(&input);

    if args.dry_run {
        if json_mode {
            println!("{}", json!({"ok": true, "command": "append", "path": args.path, "dryRun": true}));
        } else {
            println!("dry-run: would append {} bytes to {}", input.len(), args.path.display());
        }
        return Ok(());
    }

    let bytes = encode_from_string(&new_content, &existing.detected.label, existing.detected.bom)?;
    atomic_write(&args.path, &bytes)?;
    let sha = sha256_hex(&bytes);
    if json_mode {
        println!("{}", json!({"ok": true, "command": "append", "path": args.path, "bytesAppended": input.len(), "encoding": existing.detected.label, "bom": existing.meta.bom, "eol": existing.meta.eol, "sha256": sha}));
    } else {
        println!("appended {} bytes to {}", input.len(), args.path.display());
    }
    Ok(())
}
