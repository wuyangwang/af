use anyhow::Result;
use clap::Args;
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

pub fn run(_args: ReadArgs) -> Result<()> {
    todo!("af read: not implemented yet")
}
