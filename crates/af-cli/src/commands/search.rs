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
}

pub fn run(args: SearchArgs, json_mode: bool) -> Result<()> {
    let filters = SearchFilters {
        hidden: args.hidden,
        no_ignore: args.no_ignore,
        file_type: args.file_type,
        before: args.before.unwrap_or(0),
        after: args.after.unwrap_or(0),
    };
    let result = search(&args.pattern, &args.path, args.regex, &filters)?;
    if json_mode {
        println!("{}", json!({
            "ok": true,
            "command": "search",
            "pattern": args.pattern,
            "path": args.path,
            "matches": result.matches
        }));
    } else {
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
        }
        if result.matches.is_empty() {
            println!("no matches");
        }
    }
    Ok(())
}
