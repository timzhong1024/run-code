use super::Backend;
use crate::cli::ToolchainSpec;
use crate::execution::ExecutionContext;
use crate::process::{RunFailure, run_checked, run_checked_hidden, run_final};
use crate::util::{path_text, strings, write_source};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RustBackend<'a> {
    toolchain: &'a ToolchainSpec,
    packages: &'a [String],
}
impl<'a> RustBackend<'a> {
    pub fn new(toolchain: &'a ToolchainSpec, packages: &'a [String]) -> Self {
        Self {
            toolchain,
            packages,
        }
    }
}
impl Backend for RustBackend<'_> {
    fn prepare(
        &self,
        dir: &Path,
        code: &str,
        arguments: &[String],
        execution: &ExecutionContext,
        quiet: bool,
    ) -> Result<i32, RunFailure> {
        let version = &self.toolchain.version;
        let mut init = strings(&["run", "--install"]);
        init.push(version.clone());
        init.extend(strings(&[
            "cargo",
            "init",
            "--bin",
            "--name",
            "run_code_snippet",
            "--vcs",
            "none",
        ]));
        init.push(path_text(dir));
        run_checked_hidden("initialize Cargo project", "rustup", &init, None, &[])?;
        let source = dir.join("src/main.rs");
        write_source(&source, code)?;
        for package in self.packages {
            let package = RustPackageSpec::parse(package)?;
            let mut add = strings(&["run", "--install"]);
            add.push(version.clone());
            add.extend(strings(&["cargo", "add"]));
            add.push(package.dependency);
            if !package.features.is_empty() {
                add.extend(strings(&["--features"]));
                add.push(package.features.join(","));
            }
            run_checked(
                "install Rust dependencies",
                "rustup",
                &add,
                Some(dir),
                &[],
                quiet,
            )?;
        }
        let cache = target_cache()?;
        let cache_text = path_text(&cache);
        let env = [("CARGO_TARGET_DIR".into(), cache_text.clone())];
        let mut args = strings(&["run", "--install"]);
        args.push(version.clone());
        args.extend(strings(&["cargo", "run", "--quiet"]));
        args.extend(strings(&["--manifest-path"]));
        args.push(path_text(&dir.join("Cargo.toml")));
        if !arguments.is_empty() {
            args.push("--".into());
            args.extend(arguments.iter().cloned());
        }
        let result = run_final(
            "rustup",
            &args,
            Some(execution.cwd_or(dir)),
            &env,
            execution.environment(),
            quiet,
        )?;
        Ok(result.exit_code.unwrap_or(1))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RustPackageSpec {
    dependency: String,
    features: Vec<String>,
}

impl RustPackageSpec {
    fn parse(value: &str) -> Result<Self, RunFailure> {
        let (dependency, features) = if value.ends_with(']') {
            let open = value.rfind('[').ok_or_else(|| invalid_package(value))?;
            let dependency = &value[..open];
            let feature_list = &value[open + 1..value.len() - 1];
            if dependency.is_empty() || dependency.contains(['[', ']']) || feature_list.is_empty() {
                return Err(invalid_package(value));
            }
            let features = feature_list
                .split(',')
                .map(str::trim)
                .map(|feature| {
                    if feature.is_empty() || feature.contains(['[', ']']) {
                        Err(invalid_package(value))
                    } else {
                        Ok(feature.to_string())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            (dependency, features)
        } else {
            if value.is_empty() || value.contains(['[', ']']) {
                return Err(invalid_package(value));
            }
            (value, Vec::new())
        };

        Ok(Self {
            dependency: dependency.to_string(),
            features,
        })
    }
}

fn invalid_package(value: &str) -> RunFailure {
    RunFailure::message(format!(
        "invalid Rust package specification {value:?}; expected NAME[@VERSION][FEATURE,...]"
    ))
}

fn target_cache() -> Result<PathBuf, RunFailure> {
    let dirs = ProjectDirs::from("dev", "timzhong", "run-code")
        .ok_or_else(|| RunFailure::message("could not determine the user cache directory"))?;
    let path = dirs.cache_dir().join("cargo-target");
    fs::create_dir_all(&path)
        .map_err(|e| RunFailure::message(format!("failed to create Cargo cache: {e}")))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_without_features_is_unchanged() {
        assert_eq!(
            RustPackageSpec::parse("serde_json@1").unwrap(),
            RustPackageSpec {
                dependency: "serde_json@1".into(),
                features: vec![],
            }
        );
    }

    #[test]
    fn package_features_are_parsed() {
        assert_eq!(
            RustPackageSpec::parse("reqwest@0.12[json,rustls-tls]").unwrap(),
            RustPackageSpec {
                dependency: "reqwest@0.12".into(),
                features: vec!["json".into(), "rustls-tls".into()],
            }
        );
    }

    #[test]
    fn malformed_feature_lists_are_rejected() {
        for value in [
            "tokio@1[]",
            "tokio@1[full",
            "tokio@1[full,]",
            "tokio@1[full][macros]",
        ] {
            assert!(RustPackageSpec::parse(value).is_err(), "{value}");
        }
    }
}
