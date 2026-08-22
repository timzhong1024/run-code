#[cfg(test)]
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

pub const DEFAULT_PYTHON_TOOLCHAIN: &str = "3.14";
pub const DEFAULT_NODE_TOOLCHAIN: &str = "latest";
pub const DEFAULT_RUST_TOOLCHAIN: &str = "stable";
pub const DEFAULT_GO_TOOLCHAIN: &str = "latest";
pub const DEFAULT_DOTNET_TOOLCHAIN: &str = "10";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolchainKind {
    Python,
    Node,
    Rust,
    Go,
    Dotnet,
}

#[derive(Clone)]
pub struct ToolchainSpec {
    pub kind: ToolchainKind,
    pub version: String,
}

impl FromStr for ToolchainSpec {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (name, requested_version) = match value.split_once('@') {
            Some((name, version)) if !name.is_empty() && !version.is_empty() => {
                (name, Some(version))
            }
            Some(_) => return Err(format!("invalid toolchain specification: {value}")),
            None => (value, None),
        };
        let kind = match name {
            "python" | "py" => ToolchainKind::Python,
            "node" | "javascript" | "typescript" => ToolchainKind::Node,
            "rust" => ToolchainKind::Rust,
            "go" => ToolchainKind::Go,
            "dotnet" | "csharp" | "cs" => ToolchainKind::Dotnet,
            _ => {
                return Err(format!(
                    "unsupported toolchain {name:?}; expected python, node, javascript, typescript, rust, go, or dotnet"
                ));
            }
        };
        let default = match kind {
            ToolchainKind::Python => DEFAULT_PYTHON_TOOLCHAIN,
            ToolchainKind::Node => DEFAULT_NODE_TOOLCHAIN,
            ToolchainKind::Rust => DEFAULT_RUST_TOOLCHAIN,
            ToolchainKind::Go => DEFAULT_GO_TOOLCHAIN,
            ToolchainKind::Dotnet => DEFAULT_DOTNET_TOOLCHAIN,
        };
        Ok(Self {
            kind,
            version: requested_version.unwrap_or(default).to_string(),
        })
    }
}

#[derive(Parser)]
#[command(
    name = "run-code",
    version,
    about = "Run a code snippet with a selected toolchain and temporary dependencies"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Runtime/compiler in NAME or NAME@VERSION form
    #[arg(value_name = "TOOLCHAIN[@VERSION]")]
    pub toolchain: Option<ToolchainSpec>,

    /// Source file to copy into an isolated template project; reads stdin when omitted
    #[arg(value_name = "FILE")]
    pub source: Option<PathBuf>,

    /// Arguments passed to the code snippet
    #[arg(last = true, value_name = "ARG")]
    pub args: Vec<String>,

    /// Working directory for the final code process
    #[arg(long, value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// Load environment variables for the final code process
    #[arg(long, value_name = "FILE")]
    pub env_file: Option<PathBuf>,

    /// Dependency specification; Python supports NAME==VERSION
    #[arg(short = 'p', long = "package", value_name = "SPEC")]
    pub packages: Vec<String>,

    /// Run Node code as CommonJS instead of the default ESM
    #[arg(long)]
    pub commonjs: bool,

    /// Delete the generated project after the first run
    #[arg(long)]
    pub clean: bool,

    /// Only print stdout and stderr from the final code process
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print the installable Codex skill to stdout
    Skill,
}

impl Cli {
    pub fn toolchain(&self) -> &ToolchainSpec {
        self.toolchain
            .as_ref()
            .expect("toolchain is validated before execution")
    }

    pub fn validation_error(&self) -> Option<String> {
        if self.command.is_some() {
            if self.toolchain.is_some()
                || !self.packages.is_empty()
                || self.commonjs
                || self.clean
                || self.quiet
                || self.source.is_some()
                || !self.args.is_empty()
                || self.cwd.is_some()
                || self.env_file.is_some()
            {
                Some("the skill command does not accept execution arguments".into())
            } else {
                None
            }
        } else if self.toolchain.is_none() {
            Some("missing required TOOLCHAIN[@VERSION] or the skill command".into())
        } else if self.commonjs && self.toolchain().kind != ToolchainKind::Node {
            Some("--commonjs is only valid with the node toolchain".into())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_toolchain(input: &str, kind: ToolchainKind, version: &str) {
        let spec = input.parse::<ToolchainSpec>().unwrap();
        assert_eq!(spec.kind, kind);
        assert_eq!(spec.version, version);
    }

    #[test]
    fn cli_surface_stays_small() {
        let command = Cli::command();
        let visible = command
            .get_arguments()
            .filter(|arg| arg.get_id() != "help" && arg.get_id() != "version")
            .count();
        assert_eq!(visible, 9);
    }

    #[test]
    fn positional_toolchain_accepts_a_version() {
        let cli = Cli::try_parse_from(["run-code", "node@20", "--commonjs"]).unwrap();
        assert_eq!(cli.toolchain().kind, ToolchainKind::Node);
        assert_eq!(cli.toolchain().version, "20");
        assert!(cli.commonjs);
    }

    #[test]
    fn source_file_and_trailing_arguments_are_distinct() {
        let cli =
            Cli::try_parse_from(["run-code", "node@20", "snippet.ts", "--", "first", "--flag"])
                .unwrap();
        assert_eq!(cli.source, Some(PathBuf::from("snippet.ts")));
        assert_eq!(cli.args, ["first", "--flag"]);
    }

    #[test]
    fn stdin_can_also_receive_trailing_arguments() {
        let cli = Cli::try_parse_from(["run-code", "python", "--", "first"]).unwrap();
        assert!(cli.source.is_none());
        assert_eq!(cli.args, ["first"]);
    }

    #[test]
    fn execution_context_options_accept_paths() {
        let cli = Cli::try_parse_from([
            "run-code",
            "python",
            "--cwd",
            "data",
            "--env-file",
            "snippet.env",
        ])
        .unwrap();
        assert_eq!(cli.cwd, Some(PathBuf::from("data")));
        assert_eq!(cli.env_file, Some(PathBuf::from("snippet.env")));
    }

    #[test]
    fn skill_is_a_command_without_a_toolchain() {
        let cli = Cli::try_parse_from(["run-code", "skill"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Skill)));
        assert!(cli.validation_error().is_none());
    }

    #[test]
    fn quiet_uses_the_conventional_spelling() {
        assert!(
            Cli::try_parse_from(["run-code", "python", "--quiet"])
                .unwrap()
                .quiet
        );
        assert!(Cli::try_parse_from(["run-code", "python", "--quite"]).is_err());
    }

    #[test]
    fn omitted_versions_use_stable_policy() {
        assert_toolchain("python", ToolchainKind::Python, "3.14");
        assert_toolchain("node", ToolchainKind::Node, "latest");
        assert_toolchain("rust", ToolchainKind::Rust, "stable");
        assert_toolchain("go", ToolchainKind::Go, "latest");
        assert_toolchain("dotnet", ToolchainKind::Dotnet, "10");
    }

    #[test]
    fn csharp_aliases_select_dotnet() {
        assert_toolchain("csharp@10", ToolchainKind::Dotnet, "10");
        assert_toolchain("cs", ToolchainKind::Dotnet, "10");
    }

    #[test]
    fn language_names_alias_to_node() {
        for alias in ["javascript", "typescript"] {
            assert_toolchain(&format!("{alias}@20"), ToolchainKind::Node, "20");
        }
    }

    #[test]
    fn commonjs_is_node_only() {
        let cli = Cli::try_parse_from(["run-code", "python", "--commonjs"]).unwrap();
        assert!(cli.validation_error().is_some());
    }
}
