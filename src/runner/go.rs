use super::Backend;
use crate::cli::ToolchainSpec;
use crate::execution::ExecutionContext;
use crate::process::{RunFailure, run_checked, run_checked_hidden, run_final};
use crate::util::{path_text, strings, write_source};
use std::path::{Path, PathBuf};

pub struct GoBackend<'a> {
    toolchain: &'a ToolchainSpec,
    packages: &'a [String],
}
impl<'a> GoBackend<'a> {
    pub fn new(toolchain: &'a ToolchainSpec, packages: &'a [String]) -> Self {
        Self {
            toolchain,
            packages,
        }
    }
}
impl Backend for GoBackend<'_> {
    fn prepare(
        &self,
        dir: &Path,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure> {
        let toolchain = format!("go@{}", self.toolchain.version);
        let env = [("GOWORK".into(), "off".into())];
        run_checked_hidden(
            "initialize Go module",
            "mise",
            &[
                "exec".into(),
                toolchain.clone(),
                "--".into(),
                "go".into(),
                "mod".into(),
                "init".into(),
                "run-code.local/snippet".into(),
            ],
            Some(dir),
            &env,
        )?;
        let source = dir.join("main.go");
        write_source(&source, code)?;
        if !self.packages.is_empty() {
            let mut add = vec!["exec".into(), toolchain.clone()];
            add.extend(strings(&["--", "go", "get"]));
            add.extend(self.packages.iter().cloned());
            run_checked(
                "install Go dependencies",
                "mise",
                &add,
                Some(dir),
                &env,
                quiet,
            )?;
        }
        let result = if execution.has_custom_cwd() {
            let binary = snippet_binary(dir);
            let mut build = vec!["exec".into(), toolchain.clone()];
            build.extend(strings(&["--", "go", "build", "-o"]));
            build.push(path_text(&binary));
            build.push(".".into());
            run_checked_hidden("build Go snippet", "mise", &build, Some(dir), &env)?;
            run_final(
                &path_text(&binary),
                arguments,
                Some(execution.cwd_or(dir)),
                &[],
                execution.environment(),
                quiet,
            )?
        } else {
            let mut args = vec!["exec".into(), toolchain.clone()];
            args.extend(strings(&["--", "go", "run", "."]));
            args.extend(arguments.iter().cloned());
            run_final(
                "mise",
                &args,
                Some(dir),
                &env,
                execution.environment(),
                quiet,
            )?
        };
        Ok(result.exit_code.unwrap_or(1))
    }
}

fn snippet_binary(dir: &Path) -> PathBuf {
    dir.join(if cfg!(windows) {
        "run-code-snippet.exe"
    } else {
        "run-code-snippet"
    })
}
