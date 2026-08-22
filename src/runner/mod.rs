mod dotnet;
mod go;
mod node;
mod python;
mod rust;

use crate::cli::{Cli, ToolchainKind};
use crate::process::RunFailure;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

trait Backend {
    fn runs_without_project(&self) -> bool {
        false
    }
    fn run_direct(&self, _code: &str, _quiet: bool) -> Result<i32, RunFailure> {
        Err(RunFailure::message(
            "this backend does not support direct execution",
        ))
    }
    fn prepare(&self, project_dir: &Path, code: &str, quiet: bool) -> Result<i32, RunFailure>;
}

fn backend_for(cli: &Cli) -> Result<Box<dyn Backend + '_>, String> {
    let toolchain = cli.toolchain();
    match toolchain.kind {
        ToolchainKind::Python | ToolchainKind::Rust | ToolchainKind::Go | ToolchainKind::Dotnet
            if cli.commonjs =>
        {
            Err("--commonjs is only valid with the node toolchain".into())
        }
        ToolchainKind::Python => Ok(Box::new(python::PythonBackend::new(
            toolchain,
            &cli.packages,
        ))),
        ToolchainKind::Node => Ok(Box::new(node::NodeBackend::new(
            toolchain,
            &cli.packages,
            cli.commonjs,
        ))),
        ToolchainKind::Rust => Ok(Box::new(rust::RustBackend::new(toolchain, &cli.packages))),
        ToolchainKind::Go => Ok(Box::new(go::GoBackend::new(toolchain, &cli.packages))),
        ToolchainKind::Dotnet => Ok(Box::new(dotnet::DotnetBackend::new(
            toolchain,
            &cli.packages,
        ))),
    }
}

#[derive(Debug)]
pub struct RunError {
    pub message: String,
    pub hint: Option<String>,
    pub exit_code: Option<i32>,
}

impl From<RunFailure> for RunError {
    fn from(error: RunFailure) -> Self {
        let hint = error
            .missing_program
            .as_deref()
            .and_then(crate::util::program_install_hint);
        Self {
            message: error.message,
            hint,
            exit_code: error.exit_code,
        }
    }
}

pub fn run_snippet(cli: &Cli, code: &str) -> Result<i32, RunError> {
    let backend = backend_for(cli).map_err(|message| RunError {
        message,
        hint: None,
        exit_code: Some(2),
    })?;
    if backend.runs_without_project() {
        return backend.run_direct(code, cli.quiet).map_err(Into::into);
    }

    let temp_dir = Builder::new()
        .prefix("run-code-")
        .tempdir()
        .map_err(|error| RunError {
            message: format!("failed to allocate a system temporary directory: {error}"),
            hint: Some("Check that the operating system temporary directory is writable.".into()),
            exit_code: None,
        })?;
    let (project_dir, _guard): (PathBuf, Option<TempDir>) = if cli.clean {
        (temp_dir.path().to_path_buf(), Some(temp_dir))
    } else {
        (temp_dir.keep(), None)
    };

    backend
        .prepare(&project_dir, code, cli.quiet)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn commonjs_is_node_only() {
        let cli = Cli::try_parse_from(["run-code", "python", "--commonjs"]).unwrap();
        assert!(backend_for(&cli).is_err());
    }

    #[test]
    fn python_and_node_only_need_projects_for_user_packages() {
        for toolchain in ["python", "node"] {
            let direct = Cli::try_parse_from(["run-code", toolchain]).unwrap();
            assert!(backend_for(&direct).unwrap().runs_without_project());

            let with_package =
                Cli::try_parse_from(["run-code", toolchain, "--package", "example"]).unwrap();
            assert!(!backend_for(&with_package).unwrap().runs_without_project());
        }
    }
}
