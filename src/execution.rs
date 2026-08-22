use crate::cli::Cli;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct ExecutionContext {
    working_directory: Option<PathBuf>,
    environment: Vec<(String, String)>,
}

impl ExecutionContext {
    pub fn load(cli: &Cli) -> Result<Self, String> {
        let working_directory = cli.cwd.as_deref().map(canonical_directory).transpose()?;
        let environment = cli
            .env_file
            .as_deref()
            .map(read_env_file)
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            working_directory,
            environment,
        })
    }

    pub fn cwd_or<'a>(&'a self, fallback: &'a Path) -> &'a Path {
        self.working_directory.as_deref().unwrap_or(fallback)
    }

    pub fn has_custom_cwd(&self) -> bool {
        self.working_directory.is_some()
    }

    pub fn environment(&self) -> &[(String, String)] {
        &self.environment
    }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    let resolved = path.canonicalize().map_err(|error| {
        format!(
            "failed to resolve working directory {}: {error}",
            path.display()
        )
    })?;
    if !resolved.is_dir() {
        return Err(format!(
            "working directory is not a directory: {}",
            path.display()
        ));
    }
    Ok(resolved)
}

fn read_env_file(path: &Path) -> Result<Vec<(String, String)>, String> {
    let entries = dotenvy::from_path_iter(path).map_err(|error| {
        format!(
            "failed to read environment file {}: {error}",
            path.display()
        )
    })?;
    entries
        .map(|entry| {
            entry.map_err(|error| {
                format!(
                    "failed to parse environment file {}: {error}",
                    path.display()
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;

    #[test]
    fn loads_quoted_and_escaped_environment_values_without_mutating_process() {
        let process_value_before = std::env::var_os("RUN_CODE_CONTEXT_TEST");
        let dir = tempfile::tempdir().unwrap();
        let env_file = dir.path().join("snippet.env");
        fs::write(
            &env_file,
            "RUN_CODE_CONTEXT_TEST='hello world'\nESCAPED=hello\\ world\n",
        )
        .unwrap();
        let cli = Cli::try_parse_from([
            "run-code",
            "python",
            "--cwd",
            dir.path().to_str().unwrap(),
            "--env-file",
            env_file.to_str().unwrap(),
        ])
        .unwrap();

        let context = ExecutionContext::load(&cli).unwrap();
        assert_eq!(
            context.cwd_or(Path::new("fallback")),
            dir.path().canonicalize().unwrap()
        );
        assert_eq!(
            context.environment(),
            [
                ("RUN_CODE_CONTEXT_TEST".into(), "hello world".into()),
                ("ESCAPED".into(), "hello world".into())
            ]
        );
        assert_eq!(
            std::env::var_os("RUN_CODE_CONTEXT_TEST"),
            process_value_before
        );
    }

    #[test]
    fn rejects_missing_or_non_directory_working_directories() {
        let missing =
            Cli::try_parse_from(["run-code", "python", "--cwd", "run-code-missing-directory"])
                .unwrap();
        assert!(ExecutionContext::load(&missing).is_err());

        let file = tempfile::NamedTempFile::new().unwrap();
        let not_directory =
            Cli::try_parse_from(["run-code", "python", "--cwd", file.path().to_str().unwrap()])
                .unwrap();
        assert!(ExecutionContext::load(&not_directory).is_err());
    }
}
