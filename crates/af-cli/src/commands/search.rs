use af_render::json as rjson;
use af_search::filters::SearchFilters;
use af_search::search::search;
use anyhow::Result;
use clap::Args;
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Pattern to search for
    pub pattern: String,
    /// File or directory to search
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Treat pattern as a regular expression
    #[arg(long)]
    pub regex: bool,
    /// File type filter (e.g. ts, rs)
    #[arg(long = "type")]
    pub file_type: Option<String>,
    /// Glob pattern filter (e.g. "*.rs")
    #[arg(long)]
    pub glob: Option<String>,
    /// Lines of context before match
    #[arg(long)]
    pub before: Option<usize>,
    /// Lines of context after match
    #[arg(long)]
    pub after: Option<usize>,
    /// Include hidden files
    #[arg(long)]
    pub hidden: bool,
    /// Disable ignore-file filtering
    #[arg(long)]
    pub no_ignore: bool,
    /// Case-insensitive matching
    #[arg(short = 'i', long)]
    pub ignore_case: bool,
}

pub fn run(args: SearchArgs, json_mode: bool) -> Result<()> {
    af_core::path_safety::ensure_safe_path(&args.path).map_err(|e| anyhow::anyhow!(e))?;
    let filters = SearchFilters {
        hidden: args.hidden,
        no_ignore: args.no_ignore,
        file_type: args.file_type,
        glob: args.glob,
        before: args.before.unwrap_or(0),
        after: args.after.unwrap_or(0),
        ignore_case: args.ignore_case,
    };
    let result = search(&args.pattern, &args.path, args.regex, &filters)?;
    if json_mode {
        let out = json!({
            "ok": true,
            "command": "search",
            "pattern": args.pattern,
            "path": args.path.to_string_lossy(),
            "matches": result.matches
        });
        println!("{}", rjson::success(out));
    } else {
        let mut last_file = String::new();
        for m in &result.matches {
            for b in &m.before {
                println!("{}-{}-{}", m.path, b.line_number, b.text);
            }
            println!("{}:{}:{}:{}", m.path, m.line_number, m.column, m.line);
            for a in &m.after {
                println!("{}-{}-{}", m.path, a.line_number, a.text);
            }
            if !m.before.is_empty() || !m.after.is_empty() {
                println!("--");
            }
            last_file = m.path.clone();
        }
        if result.matches.is_empty() {
            println!("no matches");
        }
        let _ = last_file;
    }
    Ok(())
}
