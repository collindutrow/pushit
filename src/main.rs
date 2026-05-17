mod cli;
mod commands;
mod config;
mod error;
mod paths;
mod profile;
mod service;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Send(args) => commands::send::run(args),
        Command::Profile { command } => commands::profile::run(command),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
