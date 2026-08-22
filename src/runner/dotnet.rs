use super::Backend;
use crate::cli::ToolchainSpec;
use crate::process::{RunFailure, run_final};
use crate::util::{strings, write_source};
use std::path::Path;

pub struct DotnetBackend<'a> {
    toolchain: &'a ToolchainSpec,
    packages: &'a [String],
}

impl<'a> DotnetBackend<'a> {
    pub fn new(toolchain: &'a ToolchainSpec, packages: &'a [String]) -> Self {
        Self {
            toolchain,
            packages,
        }
    }
}

impl Backend for DotnetBackend<'_> {
    fn prepare(&self, dir: &Path, code: &str, quiet: bool) -> Result<i32, RunFailure> {
        let source = source_with_packages(code, self.packages)?;
        write_source(&dir.join("snippet.cs"), &source)?;

        let toolchain = format!("dotnet@{}", self.toolchain.version);
        let mut args = vec!["exec".into(), toolchain];
        args.extend(strings(&["--", "dotnet", "run", "snippet.cs"]));
        let result = run_final("mise", &args, Some(dir), &[], quiet)?;
        Ok(result.exit_code.unwrap_or(1))
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
