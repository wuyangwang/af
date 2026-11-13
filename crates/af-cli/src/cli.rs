use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Debug, Parser)]
#[command(
    name = "af",
    version,
    about = "Agent Files - safe file ops and search for AI coding agents",
    long_about = "AF provides predictable, encoding-safe file read/write/append/search/patch operations.\n\nAgent tips:\n  - Prefer --json for machine parsing (includes schemaVersion and stable error.code).\n  - Use `af read --meta` and `--expected-hash` to avoid concurrent overwrite conflicts.\n  - Use `--dry-run` before modifying files.\n  - Typical flow: search -> read/read --meta -> patch or write/append."
)]
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
    #[command(long_about = "Read a text file with encoding/newline detection.\nExamples:\n  af read src/main.rs --start 1 --end 120\n  af read src/main.rs --meta --json\n  af read legacy.txt --encoding gbk --plain")]
    Read(crate::commands::read::ReadArgs),
    /// Create or overwrite a file
    #[command(long_about = "Create or overwrite a file.\nExamples:\n  af write notes.txt --from input.txt --force\n  af write notes.txt --stdin --force --dry-run --json\n  af write out.txt --from in.txt --force --normalize")]
    Write(crate::commands::write::WriteArgs),
    /// Append content to a file
    #[command(long_about = "Append content to an existing file or create it if missing.\nExamples:\n  af append log.txt --from chunk.txt\n  af append log.txt --stdin --ensure-newline\n  af append log.txt --from chunk.txt --expected-hash <sha256> --json")]
    Append(crate::commands::append::AppendArgs),
    /// Search text in files or directories
    #[command(long_about = "Search text using ignore-aware traversal and ripgrep crates.\nExamples:\n  af search TODO src --type rs\n  af search \"fn main\" . --regex --glob \"*.rs\"\n  af search config . --before 2 --after 2 --json")]
    Search(crate::commands::search::SearchArgs),
    /// Apply exact replacements or unified diffs
    #[command(long_about = "Patch a file using JSON operations or a unified diff.\nExamples:\n  af patch src/lib.rs --spec change.json --dry-run\n  af patch src/lib.rs --spec change.json --expected-hash <sha256> --json\n  af patch src/lib.rs --stdin < fix.diff")]
    Patch(crate::commands::patch::PatchArgs),
    /// Print agent-oriented usage guidance and capabilities
    #[command(long_about = "Show guidance for AI agents and automation.\nExamples:\n  af agent\n  af agent --json")]
    Agent(crate::commands::agent::AgentArgs),
}
