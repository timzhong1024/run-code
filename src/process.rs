use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug)]
pub struct ProcessResult {
    pub success: bool,
    pub exit_code: Option<i32>,
}

#[derive(Debug)]
pub struct RunFailure {
    pub message: String,
    pub exit_code: Option<i32>,
    pub missing_program: Option<String>,
}

impl RunFailure {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: None,
            missing_program: None,
        }
    }

    fn start(program: &str, error: io::Error) -> Self {
        Self {
            message: format!("failed to start {program}: {error}"),
            exit_code: None,
            missing_program: (error.kind() == io::ErrorKind::NotFound).then(|| program.to_string()),
        }
    }

    fn process(stage: &str, process: ProcessResult) -> Self {
        Self {
            message: format!("{stage} failed"),
            exit_code: process.exit_code,
            missing_program: None,
        }
    }
}

pub fn run_checked_hidden(
    stage: &str,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Result<ProcessResult, RunFailure> {
    let result = run_setup(program, args, cwd, env, true)?;
    if result.success {
        Ok(result)
    } else {
        Err(RunFailure::process(stage, result))
    }
}

pub fn run_checked(
    stage: &str,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    quiet: bool,
) -> Result<ProcessResult, RunFailure> {
    let result = run_setup(program, args, cwd, env, quiet)?;
    if result.success {
        Ok(result)
    } else {
        Err(RunFailure::process(stage, result))
    }
}

pub fn run_final(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    unprinted_env: &[(String, String)],
    quiet: bool,
) -> Result<ProcessResult, RunFailure> {
    if !quiet {
        eprintln!("+ {}", display_command(program, args, cwd, env));
    }
    let status = command(program, args, cwd, env, unprinted_env)
        .status()
        .map_err(|error| RunFailure::start(program, error))?;
    Ok(ProcessResult {
        success: status.success(),
        exit_code: status.code(),
    })
}

fn run_setup(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    quiet: bool,
) -> Result<ProcessResult, RunFailure> {
    if !quiet {
        eprintln!("+ {}", display_command(program, args, cwd, env));
        let status = command(program, args, cwd, env, &[])
            .status()
            .map_err(|error| RunFailure::start(program, error))?;
        return Ok(ProcessResult {
            success: status.success(),
            exit_code: status.code(),
        });
    }

    let output = command(program, args, cwd, env, &[])
        .output()
        .map_err(|error| RunFailure::start(program, error))?;
    if !output.status.success() {
        replay_output(&output);
    }
    Ok(ProcessResult {
        success: output.status.success(),
        exit_code: output.status.code(),
    })
}

fn command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    unprinted_env: &[(String, String)],
) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    for (key, value) in unprinted_env {
        command.env(key, value);
    }
    // Explicit runner variables are applied last so isolation settings such as
    // GOWORK=off cannot be replaced by an environment file.
    for (key, value) in env {
        command.env(key, value);
    }
    command
}

fn replay_output(output: &Output) {
    let _ = io::stdout().write_all(&output.stdout);
    if output.stdout.last().is_some_and(|byte| *byte != b'\n') {
        let _ = io::stdout().write_all(b"\n");
    }
    let _ = io::stdout().flush();
    let _ = io::stderr().write_all(&output.stderr);
    if output.stderr.last().is_some_and(|byte| *byte != b'\n') {
        let _ = io::stderr().write_all(b"\n");
    }
    let _ = io::stderr().flush();
}

fn display_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> String {
    let mut parts = env
        .iter()
        .map(|(key, value)| format!("{key}={}", shell_quote(value)))
        .collect::<Vec<_>>();
    parts.push(shell_quote(program));
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    let command = parts.join(" ");
    match cwd {
        Some(cwd) => format!("cd {} && {command}", shell_quote(&cwd.to_string_lossy())),
        None => command,
    }
}

fn shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_@%+=:,./-".contains(&byte))
    {
        value.into()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displayed_command_contains_cwd_env_and_quoted_args() {
        let args = vec!["run".into(), "hello world".into()];
        let env = vec![("MODE".into(), "test value".into())];
        let shown = display_command("tool", &args, Some(Path::new("/tmp/my dir")), &env);
        assert_eq!(
            shown,
            "cd '/tmp/my dir' && MODE='test value' tool run 'hello world'"
        );
    }

    #[test]
    fn missing_program_is_reported_structurally() {
        let error = run_final(
            "run-code-program-that-does-not-exist",
            &[],
            None,
            &[],
            &[],
            true,
        )
        .unwrap_err();
        assert_eq!(
            error.missing_program.as_deref(),
            Some("run-code-program-that-does-not-exist")
        );
    }

    #[test]
    fn unprinted_environment_is_applied_but_not_displayed() {
        let shown = display_command("tool", &[], None, &[]);
        assert_eq!(shown, "tool");

        let command = command(
            "tool",
            &[],
            None,
            &[("VISIBLE".into(), "runner".into())],
            &[
                ("SECRET".into(), "hidden".into()),
                ("VISIBLE".into(), "file".into()),
            ],
        );
        assert_eq!(command.get_envs().count(), 2);
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == "VISIBLE")
                .and_then(|(_, value)| value),
            Some(std::ffi::OsStr::new("runner"))
        );
    }
}
