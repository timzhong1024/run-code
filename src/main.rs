mod cli;
mod process;
mod runner;
mod skill;
mod util;

use clap::{Parser, error::ErrorKind};
use cli::{Cli, Command};
use std::io::{self, Read};

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                let _ = error.print();
                return;
            }
            _ => return exit_with_error(&error.to_string(), 2),
        },
    };

    if let Some(message) = cli.validation_error() {
        return exit_with_error(&message, 2);
    }
    if matches!(cli.command, Some(Command::Skill)) {
        print!("{}", skill::SKILL_MD);
        return;
    }

    let mut code = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut code) {
        return exit_with_error(&format!("failed to read stdin: {error}"), 1);
    }
    if code.is_empty() {
        return exit_with_error("stdin did not contain source code", 2);
    }

    match runner::run_snippet(&cli, &code) {
        Ok(exit_code) => exit(exit_code),
        Err(error) => {
            eprintln!("run-code: {}", error.message);
            if let Some(hint) = error.hint {
                eprintln!("hint: {hint}");
            }
            exit(error.exit_code.unwrap_or(1));
        }
    }
}

fn exit_with_error(message: &str, exit_code: i32) {
    eprintln!("run-code: {message}");
    exit(exit_code);
}

fn exit(exit_code: i32) {
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}
