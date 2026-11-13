use anyhow::Result;
use clap::Args;
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct PatchArgs {
    /// Target file to patch
    pub path: PathBuf,
    /// Path to JSON patch spec
    #[arg(long)]
    pub spec: Option<PathBuf>,
    /// Read unified diff from stdin
    #[arg(long)]
    pub stdin: bool,
    /// Preview only, do not write
    #[arg(long)]
    pub dry_run: bool,
    /// Normalize output to UTF-8 no BOM + LF
    #[arg(long)]
    pub normalize: bool,
    /// Expected SHA-256 of file before patching
    #[arg(long)]
    pub expected_hash: Option<String>,
}

pub fn run(_args: PatchArgs, json_mode: bool) -> Result<()> {
    if json_mode {
        println!("{}", json!({"ok": false, "command": "patch", "error": {"code": "NOT_IMPLEMENTED", "message": "af patch not implemented yet (Phase 3)"}}));
    } else {
        println!("af patch: not implemented yet (Phase 3)");
    }
    Ok(())
}
