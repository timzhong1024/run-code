use super::Backend;
use crate::cli::ToolchainSpec;
use crate::execution::ExecutionContext;
use crate::process::{RunFailure, run_checked, run_checked_hidden, run_final};
use crate::util::{path_text, strings, write_source};
use std::path::Path;

pub struct PythonBackend<'a> {
    toolchain: &'a ToolchainSpec,
    packages: &'a [String],
}
impl<'a> PythonBackend<'a> {
    pub fn new(toolchain: &'a ToolchainSpec, packages: &'a [String]) -> Self {
        Self {
            toolchain,
            packages,
        }
    }
}

impl Backend for PythonBackend<'_> {
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
        let mut args = strings(&["run", "--quiet"]);
        args.extend(strings(&["--isolated", "--managed-python", "--python"]));
        args.push(self.toolchain.version.clone());
        args.extend(strings(&["python", "-c"]));
        args.push(code.into());
        args.extend(arguments.iter().cloned());
        let fallback = std::env::temp_dir();
        let result = run_final(
            "uv",
            &args,
            Some(execution.cwd_or(&fallback)),
            &[],
            execution.environment(),
            quiet,
        )?;
        Ok(result.exit_code.unwrap_or(1))
    }

    fn prepare(
        &self,
        dir: &Path,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure> {
        let version = &self.toolchain.version;
        let mut init = strings(&[
            "init",
            "--app",
            "--no-readme",
            "--vcs",
            "none",
            "--no-workspace",
            "--managed-python",
            "--python",
        ]);
        init.push(version.clone());
        init.push(path_text(dir));
        run_checked_hidden("initialize uv project", "uv", &init, None, &[])?;
        let source = dir.join("main.py");
        write_source(&source, code)?;
        if !self.packages.is_empty() {
            let mut args = vec!["add".into()];
            for package in self.packages {
                args.push(package_spec(package)?);
            }
            run_checked(
                "install Python dependencies",
                "uv",
                &args,
                Some(dir),
                &[],
                quiet,
            )?;
        }
        let mut args = vec!["run".into()];
        if quiet {
            args.push("--quiet".into());
        }
        args.extend(strings(&["--project"]));
        args.push(path_text(dir));
        args.extend(strings(&["--managed-python", "--python"]));
        args.push(version.clone());
        args.push("python".into());
        args.push(path_text(&source));
        args.extend(arguments.iter().cloned());
        let result = run_final(
            "uv",
            &args,
            Some(execution.cwd_or(dir)),
            &[],
            execution.environment(),
            quiet,
        )?;
        Ok(result.exit_code.unwrap_or(1))
    }
}

fn package_spec(package: &str) -> Result<String, RunFailure> {
    if let Some((name, version)) = package.split_once("==") {
        if name.is_empty() || version.is_empty() {
            return Err(RunFailure::message(format!(
                "invalid Python package specification: {package}"
            )));
        }
        Ok(package.into())
    } else if let Some((name, version)) = package.rsplit_once('@') {
        if name.is_empty() || version.is_empty() {
            return Err(RunFailure::message(format!(
                "invalid Python package specification: {package}"
            )));
        }
        Ok(format!("{name}=={version}"))
    } else {
        Ok(package.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn at_version_becomes_python_equality() {
        assert_eq!(package_spec("requests@2.32.5").unwrap(), "requests==2.32.5");
    }

    #[test]
    fn equality_version_uses_native_python_syntax() {
        assert_eq!(
            package_spec("requests==2.32.5").unwrap(),
            "requests==2.32.5"
        );
        assert_eq!(
            package_spec("httpx[socks]==0.28.1").unwrap(),
            "httpx[socks]==0.28.1"
        );
    }

    #[test]
    fn equality_version_requires_both_sides() {
        assert!(package_spec("requests==").is_err());
        assert!(package_spec("==2.32.5").is_err());
    }
}
