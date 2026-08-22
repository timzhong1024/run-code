use super::Backend;
use crate::cli::ToolchainSpec;
use crate::process::{RunFailure, run_checked, run_checked_hidden, run_final};
use crate::util::{strings, write_source};
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::Builder;

const TSX_VERSION: &str = "4.23.12";

pub struct NodeBackend<'a> {
    toolchain: &'a ToolchainSpec,
    packages: &'a [String],
    commonjs: bool,
}
impl<'a> NodeBackend<'a> {
    pub fn new(toolchain: &'a ToolchainSpec, packages: &'a [String], commonjs: bool) -> Self {
        Self {
            toolchain,
            packages,
            commonjs,
        }
    }
    fn file_name(&self) -> &'static str {
        if self.commonjs {
            "snippet.cts"
        } else {
            "snippet.ts"
        }
    }
}

impl Backend for NodeBackend<'_> {
    fn runs_without_project(&self) -> bool {
        self.packages.is_empty()
    }

    fn run_direct(&self, code: &str, quiet: bool) -> Result<i32, RunFailure> {
        let suffix = if self.commonjs { ".cts" } else { ".mts" };
        let source = Builder::new()
            .prefix("run-code-")
            .suffix(suffix)
            .tempfile()
            .map_err(|error| {
                RunFailure::message(format!(
                    "failed to allocate a temporary source file: {error}"
                ))
            })?;
        write_source(source.path(), code)?;

        let args = vec![
            "env".into(),
            "exec".into(),
            "--node".into(),
            self.toolchain.version.clone(),
            "npx".into(),
            "--yes".into(),
            format!("tsx@{TSX_VERSION}"),
            source.path().to_string_lossy().into_owned(),
        ];
        let cwd = std::env::temp_dir();
        let result = run_final("vp", &args, Some(&cwd), &[], quiet)?;
        Ok(result.exit_code.unwrap_or(1))
    }

    fn prepare(&self, dir: &Path, code: &str, quiet: bool) -> Result<i32, RunFailure> {
        let file = self.file_name();
        let manifest = serde_json::to_string_pretty(&json!({
            "name": "run-code-snippet",
            "private": true,
            "type": "module",
            "scripts": { "start": format!("tsx {file}") }
        }))
        .map_err(|e| RunFailure::message(format!("failed to build package.json: {e}")))?;
        fs::write(dir.join("package.json"), format!("{manifest}\n"))
            .map_err(|e| RunFailure::message(format!("failed to write package.json: {e}")))?;
        let source = dir.join(file);
        write_source(&source, code)?;

        let version = &self.toolchain.version;
        run_checked_hidden(
            "pin Node.js toolchain",
            "vp",
            &[
                "env".into(),
                "pin".into(),
                version.clone(),
                "--target".into(),
                "node-version".into(),
                "--force".into(),
            ],
            Some(dir),
            &[],
        )?;

        let mut install = vec![
            "env".into(),
            "exec".into(),
            "--node".into(),
            version.clone(),
            "npm".into(),
            "install".into(),
            "--save-exact".into(),
            format!("tsx@{TSX_VERSION}"),
        ];
        install.extend(self.packages.iter().cloned());
        run_checked(
            "install Node dependencies",
            "vp",
            &install,
            Some(dir),
            &[],
            quiet,
        )?;

        let final_args = if quiet {
            strings(&["exec", "tsx", file])
        } else {
            strings(&["run", "start"])
        };
        let result = run_final("vp", &final_args, Some(dir), &[], quiet)?;
        Ok(result.exit_code.unwrap_or(1))
    }
}
