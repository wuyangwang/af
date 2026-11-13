use af_core::reader::{read_text_file, read_text_range};
use af_render::json as rjson;
use anyhow::Result;
use clap::Args;
use serde_json::json;
use std::io::{self, IsTerminal};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ReadArgs {
    /// File to read
    pub path: PathBuf,
    /// Start line (1-based, inclusive)
    #[arg(long)]
    pub start: Option<usize>,
    /// End line (1-based, inclusive)
    #[arg(long)]
    pub end: Option<usize>,
    /// Print plain content without line numbers or highlighting
    #[arg(long)]
    pub plain: bool,
    /// Show line numbers
    #[arg(long)]
    pub number: bool,
    /// Output file metadata (encoding, BOM, EOL, size, hash)
    #[arg(long)]
    pub meta: bool,
    /// Explicit encoding override (e.g. utf-8, gbk)
    #[arg(long)]
    pub encoding: Option<String>,
}

pub fn run(args: ReadArgs, json_mode: bool) -> Result<()> {
    af_core::path_safety::ensure_safe_path(&args.path).map_err(|e| anyhow::anyhow!(e))?;
    if args.meta {
        let result = read_text_file(&args.path, args.encoding.as_deref()).map_err(|e| anyhow::anyhow!(e))?;
        if json_mode {
            let out = json!({
                "ok": true,
                "command": "read",
                "path": args.path.to_string_lossy(),
                "encoding": result.meta.encoding,
                "bom": result.meta.bom,
                "eol": result.meta.eol,
                "size": result.meta.size,
                "sha256": result.meta.sha256,
                "mixedNewlines": result.meta.mixed_newlines
            });
            println!("{}", rjson::success(out));
        } else {
            println!("path: {}", args.path.display());
            println!("encoding: {}", result.meta.encoding);
            println!("bom: {}", result.meta.bom);
            println!("eol: {}", result.meta.eol);
            println!("size: {}", result.meta.size);
            println!("sha256: {}", result.meta.sha256);
            println!("mixedNewlines: {}", result.meta.mixed_newlines);
        }
        return Ok(());
    }

    let ranged = read_text_range(&args.path, args.start, args.end, args.encoding.as_deref()).map_err(|e| anyhow::anyhow!(e))?;
    let content = ranged.content;
    let start_line = ranged.start_line;
    let end_line = ranged.end_line;

    if json_mode {
        let out = json!({
            "ok": true,
            "command": "read",
            "path": args.path.to_string_lossy(),
            "encoding": ranged.meta.encoding,
            "bom": ranged.meta.bom,
            "eol": ranged.meta.eol,
            "startLine": start_line,
            "endLine": end_line,
            "content": content,
            "sha256": ranged.meta.sha256
        });
        println!("{}", rjson::success(out));
        return Ok(());
    }

    let use_color = !args.plain && io::stdout().is_terminal();
    let show_numbers = args.number || !args.plain;
    let body = if use_color {
        af_render::highlight::highlight_to_ansi(&content, Some(&args.path.to_string_lossy())).unwrap_or_else(|| content.clone())
    } else {
        content.clone()
    };
    if show_numbers {
        let base = start_line;
        for (i, line) in body.lines().enumerate() {
            println!("{:>6}\t{}", base + i, line);
        }
    } else {
        print!("{}", body);
    }
    Ok(())
}
