mod dotnet;
mod go;
mod node;
mod python;
mod rust;

use crate::cli::{Cli, ToolchainKind};
use crate::execution::ExecutionContext;
use crate::process::RunFailure;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

trait Backend {
    fn runs_without_project(&self) -> bool {
        false
    }
    fn run_direct(
        &self,
        _code: &str,
        _arguments: &[String],
        _execution: &ExecutionContext,
        _quiet: bool,
    ) -> Result<i32, RunFailure> {
        Err(RunFailure::message(
            "this backend does not support direct execution",
        ))
    }
    fn prepare(
        &self,
        project_dir: &Path,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure>;
}

fn backend_for(cli: &Cli) -> Box<dyn Backend + '_> {
    let toolchain = cli.toolchain();
    match toolchain.kind {
        ToolchainKind::Python => Box::new(python::PythonBackend {
            toolchain,
            packages: &cli.packages,
        }),
        ToolchainKind::Node => Box::new(node::NodeBackend {
            toolchain,
            packages: &cli.packages,
            commonjs: cli.commonjs,
        }),
        ToolchainKind::Rust => Box::new(rust::RustBackend {
            toolchain,
            packages: &cli.packages,
        }),
        ToolchainKind::Go => Box::new(go::GoBackend {
            toolchain,
            packages: &cli.packages,
        }),
        ToolchainKind::Dotnet => Box::new(dotnet::DotnetBackend {
            toolchain,
            packages: &cli.packages,
        }),
    }
}

pub fn run_snippet(cli: &Cli, execution: &ExecutionContext, code: &str) -> Result<i32, RunFailure> {
    let backend = backend_for(cli);
    // File input is always copied into an isolated template project. It never
    // runs in, or discovers dependencies from, the source file's project.
    if should_run_direct(cli, backend.as_ref()) {
        return backend.run_direct(code, &cli.args, execution, cli.quiet);
    }

    let temp_dir = Builder::new()
        .prefix("run-code-")
        .tempdir()
        .map_err(|error| RunFailure {
            message: format!("failed to allocate a system temporary directory: {error}"),
            hint: Some("Check that the operating system temporary directory is writable.".into()),
            exit_code: None,
            missing_program: None,
        })?;
    let (project_dir, _guard): (PathBuf, Option<TempDir>) = if cli.clean {
        (temp_dir.path().to_path_buf(), Some(temp_dir))
    } else {
        (temp_dir.keep(), None)
    };

    backend.prepare(&project_dir, code, &cli.args, execution, cli.quiet)
}

fn should_run_direct(cli: &Cli, backend: &dyn Backend) -> bool {
    cli.source.is_none() && backend.runs_without_project()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn python_and_node_only_need_projects_for_user_packages() {
        for toolchain in ["python", "node"] {
            let direct = Cli::try_parse_from(["run-code", toolchain]).unwrap();
            assert!(backend_for(&direct).runs_without_project());

            let with_package =
                Cli::try_parse_from(["run-code", toolchain, "--package", "example"]).unwrap();
            assert!(!backend_for(&with_package).runs_without_project());
        }
    }

    #[test]
    fn source_files_force_an_isolated_template_project() {
        for toolchain in ["python", "node"] {
            let stdin = Cli::try_parse_from(["run-code", toolchain]).unwrap();
            let stdin_backend = backend_for(&stdin);
            assert!(should_run_direct(&stdin, stdin_backend.as_ref()));

            let cli = Cli::try_parse_from(["run-code", toolchain, "snippet.txt"]).unwrap();
            let backend = backend_for(&cli);
            assert!(!should_run_direct(&cli, backend.as_ref()));
        }
    }
}
