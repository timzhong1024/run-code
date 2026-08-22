mod cli;
mod process;
mod runner;
mod skill;
mod source;
mod util;

use clap::{Parser, error::ErrorKind};
use cli::{Cli, Command};

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

    let code = match source::read(cli.source.as_deref()) {
        Ok(code) => code,
        Err(error) => return exit_with_error(&error, 1),
    };
    if code.is_empty() {
        let input = if cli.source.is_some() {
            "source file"
        } else {
            "stdin"
        };
        return exit_with_error(&format!("{input} did not contain source code"), 2);
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
