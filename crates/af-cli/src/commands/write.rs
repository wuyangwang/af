use af_core::writer::{write_new_file, write_preserving, EditMode};
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
    /// Line ending: lf, crlf, cr
    #[arg(long)]
    pub eol: Option<String>,
    /// Normalize output to UTF-8 no BOM + LF
    #[arg(long)]
    pub normalize: bool,
    /// Preview only, do not write
    #[arg(long)]
    pub dry_run: bool,
    /// Allow overwriting an existing file
    #[arg(long)]
    pub force: bool,
}

fn read_input(args: &WriteArgs) -> Result<String> {
    if args.from.is_some() && args.stdin {
        bail!("--stdin and --from are mutually exclusive");
    }
    if let Some(from) = &args.from {
        return Ok(fs::read_to_string(from)?);
    }
    // default to stdin when --stdin or when data piped? keep simple: require --stdin if no --from
    if args.stdin || args.from.is_none() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    bail!("no input provided");
}

pub fn run(args: WriteArgs, json_mode: bool) -> Result<()> {
    if args.path.exists() && !args.force {
        bail!("target exists; pass --force to overwrite");
    }
    let input = read_input(&args)?;
    let mode = if args.normalize { EditMode::Normalize } else { EditMode::Preserve };

    if args.dry_run {
        if json_mode {
            println!("{}", json!({"ok": true, "command": "write", "path": args.path, "dryRun": true}));
        } else {
            println!("dry-run: would write {} bytes to {}", input.len(), args.path.display());
        }
        return Ok(());
    }

    let outcome = if args.path.exists() {
        write_preserving(&args.path, &input, mode)?
    } else {
        write_new_file(&args.path, &input)?
    };

    if json_mode {
        println!("{}", json!({
            "ok": true,
            "command": "write",
            "path": args.path,
            "bytesWritten": outcome.bytes_written,
            "encoding": outcome.meta.encoding,
            "bom": outcome.meta.bom,
            "eol": outcome.meta.eol,
            "sha256": outcome.meta.sha256
        }));
    } else {
        println!("wrote {} bytes to {}", outcome.bytes_written, args.path.display());
    }
    Ok(())
}
