mod cli;
mod commands;

use af_core::errors::AfError;
use clap::Parser;
use cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match commands::dispatch(cli) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            if let Some(af) = e.downcast_ref::<AfError>() {
                eprintln!("error: {af}");
                ExitCode::from(af.exit_code() as u8)
            } else {
                eprintln!("error: {e:#}");
                ExitCode::from(1)
            }
        }
    }
}
