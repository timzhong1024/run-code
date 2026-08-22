# run-code

[简体中文](README_zh.md)

Run Python, TypeScript, JavaScript, Rust, Go, or C# snippets with a selected runtime/toolchain version and temporary dependencies in an isolated environment.

## Installation

Homebrew is recommended on macOS:

```bash
brew install timzhong1024/tap/run-code
```

You can also install the prebuilt binary through npm:

```bash
npm install --global @timzhong/run-code
```

Or install from source:

```bash
cargo install --locked --git https://github.com/timzhong1024/run-code
```

GitHub Releases also provide standalone binaries for macOS, Linux, and Windows.

`run-code` delegates to external tools for each language. Install only the backends you use:

| Language | Required tool |
| --- | --- |
| Python | [uv](https://docs.astral.sh/uv/getting-started/installation/) |
| TypeScript / JavaScript | [Vite+ (`vp`)](https://viteplus.dev/guide/) |
| Rust | [rustup](https://rustup.rs/) |
| Go | [mise](https://mise.jdx.dev/getting-started.html) |
| C# / .NET | [mise](https://mise.jdx.dev/getting-started.html) |

## Examples

### TypeScript

```bash
run-code node@20 --package zod@4 --clean <<'TS'
import { z } from "zod";
console.log(await Promise.resolve(z.string().parse("hello")));
TS
```

### Python

```bash
run-code python@3.14 --package requests==2.32.5 --clean <<'PY'
import requests
print(requests.__version__)
PY
```

### Rust

```bash
run-code rust@stable --package serde_json@1 --clean <<'RS'
fn main() {
    println!("{}", serde_json::json!({"hello": "world"}));
}
RS
```

### C#

```bash
run-code dotnet@10 --package Spectre.Console@0.50.0 --clean <<'CS'
using Spectre.Console;
AnsiConsole.MarkupLine("[green]Hello from C#[/]");
CS
```

For asynchronous Rust, specify Cargo features in the dependency spec:

```bash
run-code rust@stable --package 'tokio@1[full]' --clean <<'RS'
#[tokio::main]
async fn main() {
    println!("async");
}
RS
```

In Windows PowerShell 7, use a single-quoted here-string:

```powershell
@'
print("hello\nworld")
'@ | run-code python@3.14 --clean
```

Fish does not support heredocs, so use `printf`:

```fish
printf '%s\n' \
    'const value: string = await Promise.resolve("hello");' \
    'console.log(value);' |
    run-code node@20
```

## Agent Skill

`run-code skill` prints the complete bundled `SKILL.md` from the installed binary. An agent can discover the skill by its name and description, then read the full instructions when the task matches.

Install it in the current project:

```bash
mkdir -p .agents/skills/run-code-snippet
run-code skill > .agents/skills/run-code-snippet/SKILL.md
```

Or install it in your user directory to make it available across projects:

```bash
mkdir -p ~/.agents/skills/run-code-snippet
run-code skill > ~/.agents/skills/run-code-snippet/SKILL.md
```

Codex automatically discovers skills in these directories. See the [Codex Skills documentation](https://learn.chatgpt.com/docs/build-skills) for details.

## Why this project exists

Running a temporary snippet often means paying the setup cost of creating a project, installing dependencies, and preparing an environment. Switching to a different runtime or toolchain version for one task is also cumbersome, while installing packages globally or into an existing project creates unwanted state.

Several related tools solve parts of this problem, but none matched the combination of temporary dependencies, version switching, and isolated execution needed here. Inspired by snippet runners, version managers, and temporary package executors, `run-code` combines those steps into one command.

## Security

`run-code` provides environment and dependency isolation; it is not a security sandbox. Snippets and third-party dependencies run with the current user's permissions and may access local files, the network, environment variables, and credentials.

Dependency installation may execute npm lifecycle scripts, Python build backends, Cargo `build.rs` scripts, or other ecosystem-specific build code. Run only trusted code and dependencies. Inspect unfamiliar packages before use, pin versions in sensitive environments, and avoid exposing unnecessary secrets. `--clean` removes only the temporary project; it cannot undo system or network side effects, and package-manager download caches remain in place.

Report vulnerabilities privately through GitHub private vulnerability reporting. See [SECURITY.md](SECURITY.md) for scope and reporting instructions.

## CLI reference

```text
run-code [OPTIONS] TOOLCHAIN[@VERSION]
run-code skill
```

- `TOOLCHAIN[@VERSION]`: Select a language and optional version. Supported toolchains are `python`, `node`, `rust`, `go`, and `dotnet`; `csharp` and `cs` are aliases for `dotnet`.
- `-p, --package SPEC`: Add a temporary dependency. Repeat the option to install multiple packages. Specs follow each ecosystem: Python uses `NAME==VERSION`; Node, Rust, Go, and .NET use `NAME@VERSION`. Rust also supports `NAME[@VERSION][FEATURE,...]`, such as `'tokio@1[full]'`.
- `--commonjs`: Run Node code as CommonJS. The default is ESM with top-level `await` support.
- `--clean`: Delete the generated project after execution. Without this option, the project is retained and its path appears in the displayed command.
- `--quiet`: Hide project setup, dependency installation, and command display; print only stdout/stderr from the final code process.
- `skill`: Print the bundled `run-code-snippet` skill.
- `-h, --help`: Show help.
- `-V, --version`: Show the version.

When the version is omitted, built-in defaults are used: Python 3.14, Node latest, Rust stable, Go latest, and .NET 10. C# runs as a .NET 10+ file-based app. Python and Node execute directly without creating a project when no `--package` option is provided; an isolated project is created only when dependencies are needed. Source code is read from stdin, and package-manager download caches remain enabled. By default, `run-code` displays only dependency installation and final execution commands while streaming their stdout/stderr; project initialization output is shown only when initialization fails.
