use super::Backend;
use crate::cli::ToolchainSpec;
use crate::process::{RunFailure, run_checked, run_checked_hidden, run_final};
use crate::util::{strings, write_source};
use std::path::Path;

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
    fn prepare(&self, dir: &Path, code: &str, quiet: bool) -> Result<i32, RunFailure> {
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
        let mut args = vec!["exec".into(), toolchain.clone()];
        args.extend(strings(&["--", "go", "run", "."]));
        let result = run_final("mise", &args, Some(dir), &env, quiet)?;
        Ok(result.exit_code.unwrap_or(1))
    }
}
