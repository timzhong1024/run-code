use super::Backend;
use crate::cli::ToolchainSpec;
use crate::execution::ExecutionContext;
use crate::process::{RunFailure, run_checked, run_final};
use crate::util::{path_text, strings, write_source};
use std::fs;
use std::path::Path;
use tempfile::Builder;

const TSX_VERSION: &str = "4.23.12";
const PACKAGE_JSON: &str = r#"{
  "name": "run-code-snippet",
  "private": true,
  "type": "module"
}
"#;

pub(super) struct NodeBackend<'a> {
    pub(super) toolchain: &'a ToolchainSpec,
    pub(super) packages: &'a [String],
    pub(super) commonjs: bool,
}
impl NodeBackend<'_> {
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

    fn run_direct(
        &self,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure> {
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

        let mut args = vec![
            "env".into(),
            "exec".into(),
            "--node".into(),
            self.toolchain.version.clone(),
            "npx".into(),
            "--yes".into(),
            format!("tsx@{TSX_VERSION}"),
            source.path().to_string_lossy().into_owned(),
        ];
        args.extend(arguments.iter().cloned());
        let fallback = std::env::temp_dir();
        run_final(
            "vp",
            &args,
            Some(execution.cwd_or(&fallback)),
            &[],
            execution.environment(),
            quiet,
        )
    }

    fn prepare(
        &self,
        dir: &Path,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure> {
        let file = self.file_name();
        fs::write(dir.join("package.json"), PACKAGE_JSON)
            .map_err(|e| RunFailure::message(format!("failed to write package.json: {e}")))?;
        let source = dir.join(file);
        write_source(&source, code)?;

        let version = &self.toolchain.version;
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

        let final_args = final_command(dir, &source, version, arguments);
        run_final(
            "vp",
            &final_args,
            Some(execution.cwd_or(dir)),
            &[],
            execution.environment(),
            quiet,
        )
    }
}

fn final_command(
    project_dir: &Path,
    source: &Path,
    version: &str,
    arguments: &[String],
) -> Vec<String> {
    let tsx_cli = project_dir.join("node_modules/tsx/dist/cli.mjs");
    let mut command = strings(&["env", "exec", "--node"]);
    command.push(version.into());
    command.push("node".into());
    command.push(path_text(&tsx_cli));
    command.push(path_text(source));
    command.extend(arguments.iter().cloned());
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_command_uses_explicit_toolchain_and_absolute_paths() {
        let project = Path::new("template");
        let source = project.join("snippet.ts");
        let command = final_command(project, &source, "20", &["first".into(), "--flag".into()]);
        assert_eq!(&command[..5], ["env", "exec", "--node", "20", "node"]);
        assert_eq!(
            command[5],
            path_text(&project.join("node_modules/tsx/dist/cli.mjs"))
        );
        assert_eq!(command[6], path_text(&source));
        assert_eq!(command[7], "first");
        assert_eq!(command[8], "--flag");
    }
}
