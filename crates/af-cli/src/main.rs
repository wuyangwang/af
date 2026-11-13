mod cli;
mod commands;

use af_core::errors::AfError;
use clap::Parser;
use cli::{Cli, OutputMode};
use serde_json::json;
use std::env;
use std::process::ExitCode;

fn wants_json() -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|a| a == "--json")
        || args.windows(2).any(|w| w[0] == "--output" && w[1] == "json")
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json_mode = cli.json || matches!(cli.output, OutputMode::Json) || wants_json();
    match commands::dispatch(cli) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            if let Some(af) = e.downcast_ref::<AfError>() {
                let code = af.exit_code() as u8;
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "ok": false,
                            "error": {
                                "code": format!("E{}", code),
                                "message": af.to_string()
                            }
                        })
                    );
                } else {
                    eprintln!("error: {af}");
                }
                ExitCode::from(code)
            } else {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "ok": false,
                            "error": {
                                "code": "E1",
                                "message": format!("{e:#}")
                            }
                        })
                    );
                } else {
                    eprintln!("error: {e:#}");
                }
                ExitCode::from(1)
            }
        }
    }
}
