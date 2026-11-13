pub mod append;
pub mod patch;
pub mod read;
pub mod search;
pub mod write;

use crate::cli::Cli;
use anyhow::Result;

pub fn dispatch(cli: Cli) -> Result<()> {
    let _json = cli.json || matches!(cli.output, crate::cli::OutputMode::Json);
    match cli.command {
        crate::cli::Commands::Read(args) => read::run(args),
        crate::cli::Commands::Write(args) => write::run(args),
        crate::cli::Commands::Append(args) => append::run(args),
        crate::cli::Commands::Search(args) => search::run(args),
        crate::cli::Commands::Patch(args) => patch::run(args),
    }
}
