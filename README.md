# run-code

[简体中文](README_zh.md)

Run a disposable Python, TypeScript/JavaScript, Rust, Go, or C# snippet with a selected runtime version and temporary dependencies—without modifying your global environment or current project.

```bash
echo 'print("hello")' | run-code python@3.14
```

## Why run-code exists

A small experiment should not require creating a project, choosing a package manager layout, switching the active runtime, installing dependencies, and deleting everything afterward. Installing a package globally or into the current project is faster initially, but leaves unrelated state behind.

`run-code` turns the disposable case into one command:

1. select a runtime or toolchain version;
2. prepare dependencies in an isolated temporary environment;
3. run code from stdin or a source file.

Use it to try a package from its README, verify behavior on a specific runtime, reproduce a small example, or let an agent run a focused check. Use a normal project for multi-file programs, durable dependencies, or build configuration. `run-code` is isolation for convenience, not a security sandbox.

## For agents

After installing the binary, an agent can read its matching `run-code-snippet` skill and exact version-specific instructions by running:

```bash
run-code skill
```

Install the skill into a project when its use should be repository-specific:

```bash
mkdir -p .agents/skills/run-code-snippet
run-code skill > .agents/skills/run-code-snippet/SKILL.md
```

Or install it globally for reuse across projects:

```bash
mkdir -p ~/.agents/skills/run-code-snippet
run-code skill > ~/.agents/skills/run-code-snippet/SKILL.md
```

For a one-off check, the essential pattern is:

```bash
run-code TOOLCHAIN[@VERSION] [--package SPEC ...] [--clean] [--quiet] <<'LANG'
CODE
LANG
```

This gives the agent an explicit runtime, disposable dependencies, isolated project state, and normal stdout/stderr without asking it to scaffold a project manually.

## Install

On macOS:

```bash
brew install timzhong1024/tap/run-code
```

Other options:

```bash
npm install --global @timzhong2000/run-code
cargo install --locked --git https://github.com/timzhong1024/run-code
```

Standalone macOS, Linux, and Windows binaries are available from [GitHub Releases](https://github.com/timzhong1024/run-code/releases).

`run-code` delegates runtime installation to existing tools. Install only the backends you use:

| Code | Toolchain argument | Required backend |
| --- | --- | --- |
| Python | `python[@VERSION]` | [uv](https://docs.astral.sh/uv/getting-started/installation/) |
| TypeScript / JavaScript | `node[@VERSION]` | [Vite+ (`vp`)](https://viteplus.dev/guide/) |
| Rust | `rust[@VERSION]` | [rustup](https://rustup.rs/) |
| Go | `go[@VERSION]` | [mise](https://mise.jdx.dev/getting-started.html) |
| C# | `dotnet[@VERSION]` | [mise](https://mise.jdx.dev/getting-started.html) |

## Quick examples

### TypeScript with an npm package

```bash
run-code node@20 --package zod@4 --clean <<'TS'
import { z } from "zod";
console.log(await Promise.resolve(z.object({ id: z.number() }).parse({ id: 1 })));
TS
```

### Python with a PyPI package

```bash
run-code python@3.14 --package requests==2.32.5 --clean <<'PY'
import requests
print(requests.__version__)
PY
```

### Rust with a crate

```bash
run-code rust@stable --package serde_json@1 --clean <<'RS'
fn main() {
    println!("{}", serde_json::json!({"hello": "world"}));
}
RS
```

### Source files, arguments, working directory, and environment

```bash
run-code node@20 \
  --package zod@4 \
  --cwd ./fixtures \
  --env-file ./snippet.env \
  snippet.ts -- first --verbose
```

The file is copied into a fresh isolated template; its existing project, dependencies, and sibling files are not used. Arguments after `--` go to the snippet. `--cwd` affects only the final code process, while `--env-file` supplies dotenv-compatible variables without changing the current shell or exposing their values in the displayed command.

## Execution behavior

- Omitting a version selects the built-in stable policy: Python 3.14, Node latest, Rust stable, Go latest, and .NET 10.
- Python and Node stdin snippets without `--package` run directly; dependencies or file input use an isolated template project.
- Temporary projects are retained by default so their generated command paths remain inspectable. Add `--clean` for a one-off run.
- Package-manager download caches remain enabled, so isolation does not mean downloading every package again.
- By default, dependency installation and the final command are displayed and their output is streamed. `--quiet` leaves only the final process stdout/stderr.
- Node defaults to ESM with TypeScript and top-level `await` support. Use `--commonjs` only for CommonJS-specific code.

## CLI reference

```text
run-code [OPTIONS] TOOLCHAIN[@VERSION]
run-code [OPTIONS] TOOLCHAIN[@VERSION] FILE [-- ARG ...]
run-code skill
```

| Argument | Meaning |
| --- | --- |
| `TOOLCHAIN[@VERSION]` | `python`, `node`, `rust`, `go`, or `dotnet`; `javascript`/`typescript` alias `node`, and `csharp`/`cs` alias `dotnet` |
| `FILE` | Copy and run one source file instead of reading stdin |
| `-- ARG ...` | Pass trailing arguments to the snippet |
| `-p, --package SPEC` | Add a dependency; repeat for multiple packages |
| `--cwd DIR` | Set the final snippet process working directory |
| `--env-file FILE` | Load dotenv variables for the final process |
| `--commonjs` | Run Node code as CommonJS instead of ESM |
| `--clean` | Remove the temporary project after execution |
| `--quiet` | Show only final-process stdout/stderr |
| `skill` | Print the bundled agent skill |

Package specs follow their ecosystems. Python accepts `NAME==VERSION`; Node, Rust, Go, and .NET accept `NAME@VERSION`. Rust features use `NAME[@VERSION][FEATURE,...]`, for example `'tokio@1[full]'`.

### Shell input

Bash and Zsh use the quoted heredocs shown above. Fish can pipe `printf`:

```fish
printf '%s\n' \
    'const value: string = await Promise.resolve("hello");' \
    'console.log(value);' |
    run-code node@20
```

PowerShell 7 can pipe a single-quoted here-string:

```powershell
@'
print("hello\nworld")
'@ | run-code python@3.14 --clean
```

## Security

Snippets, dependencies, package lifecycle hooks, Python build backends, and Cargo build scripts run with the current user's permissions. They may access files, the network, environment variables, credentials, and other processes. Run only trusted code and packages; pin versions when reproducibility matters and do not pass secrets to untrusted snippets.

`--clean` removes the temporary project, but cannot undo filesystem or network side effects. See [SECURITY.md](SECURITY.md) for the reporting policy and complete security boundary.
