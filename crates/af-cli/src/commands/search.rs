use anyhow::Result;
use clap::Args;
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

pub fn run(_args: SearchArgs) -> Result<()> {
    todo!("af search: not implemented yet")
}
