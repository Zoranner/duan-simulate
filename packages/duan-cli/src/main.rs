mod commands;
mod error;

use std::process::ExitCode;

use clap::Parser;

use crate::commands::Cli;

fn main() -> ExitCode {
    match commands::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
