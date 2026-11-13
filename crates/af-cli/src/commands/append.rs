use anyhow::Result;
use clap::Args;
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

pub fn run(_args: AppendArgs) -> Result<()> {
    todo!("af append: not implemented yet")
}
