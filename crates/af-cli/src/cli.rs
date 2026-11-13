use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "af", version, about = "Agent Files - safe file ops and search for AI coding agents")]
pub struct Cli {
    /// Output mode: human-readable or machine JSON
    #[arg(long, global = true, value_enum, default_value_t = OutputMode::Human)]
    pub output: OutputMode,

    /// Shorthand for --output json
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Read file content, ranges, or metadata
    Read(crate::commands::read::ReadArgs),
    /// Create or overwrite a file
    Write(crate::commands::write::WriteArgs),
    /// Append content to a file
    Append(crate::commands::append::AppendArgs),
    /// Search text in files or directories
    Search(crate::commands::search::SearchArgs),
    /// Apply exact replacements or unified diffs
    Patch(crate::commands::patch::PatchArgs),
}
