use af_core::reader::read_text_file;
use anyhow::Result;
use clap::Args;
use serde_json::json;
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

fn slice_lines(content: &str, start: Option<usize>, end: Option<usize>) -> (String, usize, usize) {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let total = if content.is_empty() { 0 } else { lines.len() };
    let s = start.unwrap_or(1).max(1);
    let e = end.unwrap_or(total.max(1)).min(total.max(1));
    if total == 0 {
        return (String::new(), 1, 0);
    }
    let s_idx = s.saturating_sub(1).min(total.saturating_sub(1));
    let e_idx = e.saturating_sub(1).min(total.saturating_sub(1));
    if s_idx > e_idx {
        return (String::new(), s, e);
    }
    let mut out = String::new();
    for line in &lines[s_idx..=e_idx] {
        out.push_str(line);
    }
    (out, s_idx + 1, e_idx + 1)
}

pub fn run(args: ReadArgs, json_mode: bool) -> Result<()> {
    let result = read_text_file(&args.path, args.encoding.as_deref()).map_err(|e| anyhow::anyhow!(e))?;
    let (content, start_line, end_line) = slice_lines(&result.content, args.start, args.end);

    if json_mode {
        if args.meta {
            let out = json!({
                "ok": true,
                "command": "read",
                "path": args.path,
                "encoding": result.meta.encoding,
                "bom": result.meta.bom,
                "eol": result.meta.eol,
                "size": result.meta.size,
                "sha256": result.meta.sha256,
                "mixedNewlines": result.meta.mixed_newlines
            });
            println!("{}", out);
        } else {
            let out = json!({
                "ok": true,
                "command": "read",
                "path": args.path,
                "encoding": result.meta.encoding,
                "bom": result.meta.bom,
                "eol": result.meta.eol,
                "startLine": start_line,
                "endLine": end_line,
                "content": content,
                "sha256": result.meta.sha256
            });
            println!("{}", out);
        }
        return Ok(());
    }

    if args.meta {
        println!("path: {}", args.path.display());
        println!("encoding: {}", result.meta.encoding);
        println!("bom: {}", result.meta.bom);
        println!("eol: {}", result.meta.eol);
        println!("size: {}", result.meta.size);
        println!("sha256: {}", result.meta.sha256);
        println!("mixedNewlines: {}", result.meta.mixed_newlines);
        return Ok(());
    }

    if args.number && !args.plain {
        let base = start_line;
        for (i, line) in content.lines().enumerate() {
            println!("{:>6}\t{}", base + i, line);
        }
    } else {
        print!("{}", content);
    }
    Ok(())
}
