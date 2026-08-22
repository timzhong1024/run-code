use super::Backend;
use crate::cli::ToolchainSpec;
use crate::execution::ExecutionContext;
use crate::process::{RunFailure, run_final};
use crate::util::{path_text, strings, write_source};
use std::path::Path;

pub(super) struct DotnetBackend<'a> {
    pub(super) toolchain: &'a ToolchainSpec,
    pub(super) packages: &'a [String],
}

impl Backend for DotnetBackend<'_> {
    fn prepare(
        &self,
        dir: &Path,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure> {
        let source = source_with_packages(code, self.packages)?;
        let source_path = dir.join("snippet.cs");
        write_source(&source_path, &source)?;

        let toolchain = format!("dotnet@{}", self.toolchain.version);
        let mut args = vec!["exec".into(), toolchain];
        args.extend(strings(&["--", "dotnet", "run"]));
        args.push(path_text(&source_path));
        if !arguments.is_empty() {
            args.push("--".into());
            args.extend(arguments.iter().cloned());
        }
        run_final(
            "mise",
            &args,
            Some(execution.cwd_or(dir)),
            &[],
            execution.environment(),
            quiet,
        )
    }
}

fn source_with_packages(code: &str, packages: &[String]) -> Result<String, RunFailure> {
    let mut source = String::new();
    for package in packages {
        if package.is_empty()
            || package.chars().any(char::is_whitespace)
            || package.starts_with('@')
            || package.ends_with('@')
        {
            return Err(RunFailure::message(format!(
                "invalid NuGet package specification {package:?}; expected NAME[@VERSION]"
            )));
        }
        source.push_str("#:package ");
        source.push_str(package);
        if !package.contains('@') {
            source.push_str("@*");
        }
        source.push('\n');
    }
    source.push_str(code);
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_directives_are_prepended_to_source() {
        let source = source_with_packages(
            "using Spectre.Console;\nAnsiConsole.WriteLine(\"hello\");\n",
            &["Spectre.Console@0.50.0".into(), "Humanizer".into()],
        )
        .unwrap();
        assert_eq!(
            source,
            "#:package Spectre.Console@0.50.0\n#:package Humanizer@*\nusing Spectre.Console;\nAnsiConsole.WriteLine(\"hello\");\n"
        );
    }

    #[test]
    fn malformed_package_directives_are_rejected() {
        for value in [
            "",
            "Spectre.Console@",
            "@0.50.0",
            "Bad Package",
            "Bad\nCode",
        ] {
            assert!(
                source_with_packages("", &[value.into()]).is_err(),
                "{value:?}"
            );
        }
    }
}
