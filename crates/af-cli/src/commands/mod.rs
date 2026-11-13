pub mod agent;
pub mod append;
pub mod patch;
pub mod read;
pub mod search;
pub mod write;

use crate::cli::{Cli, OutputMode};
use anyhow::Result;

pub fn dispatch(cli: Cli) -> Result<()> {
    let json = cli.json || matches!(cli.output, OutputMode::Json);
    match cli.command {
        crate::cli::Commands::Read(args) => read::run(args, json),
        crate::cli::Commands::Write(args) => write::run(args, json),
        crate::cli::Commands::Append(args) => append::run(args, json),
        crate::cli::Commands::Search(args) => search::run(args, json),
        crate::cli::Commands::Patch(args) => patch::run(args, json),
        crate::cli::Commands::Agent(args) => agent::run(args, json),
    }
}
