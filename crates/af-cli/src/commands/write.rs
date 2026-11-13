use anyhow::Result;
use clap::Args;
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

pub fn run(_args: WriteArgs) -> Result<()> {
    todo!("af write: not implemented yet")
}
