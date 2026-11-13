mod cli;
mod commands;

use af_core::errors::AfError;
use af_render::json as rjson;
use clap::Parser;
use cli::{Cli, OutputMode};
use std::env;
use std::process::ExitCode;

fn wants_json() -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|a| a == "--json")
        || args.windows(2).any(|w| w[0] == "--output" && w[1] == "json")
}

fn init_tracing(json_mode: bool) {
    if json_mode {
        return;
    }
    use tracing_subscriber::EnvFilter;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() -> ExitCode {
    let json_mode_early = wants_json();
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            use clap::error::ErrorKind;
            let kind = e.kind();
            if matches!(kind, ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                print!("{}", e);
                return ExitCode::from(0);
            }
            init_tracing(json_mode_early);
            let af = AfError::invalid_input(e.to_string());
            if json_mode_early {
                println!("{}", rjson::error_value(&af));
            } else {
                eprint!("{}", e);
            }
            return ExitCode::from(af.exit_code() as u8);
        }
    };
    let json_mode = cli.json || matches!(cli.output, OutputMode::Json) || json_mode_early;
    init_tracing(json_mode);
    match commands::dispatch(cli) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            if let Some(af) = e.downcast_ref::<AfError>() {
                let code = af.exit_code() as u8;
                if json_mode {
                    println!("{}", rjson::error_value(af));
                } else {
                    eprintln!("error: {af}");
                }
                ExitCode::from(code)
            } else {
                let af = AfError::general(format!("{e:#}"));
                if json_mode {
                    println!("{}", rjson::error_value(&af));
                } else {
                    eprintln!("error: {e:#}");
                }
                ExitCode::from(1)
            }
        }
    }
}
