---
name: run-code-snippet
description: 临时快捷运行一段 Python、TypeScript、JavaScript、Rust、Go 或 C# 代码时使用；支持指定 runtime/toolchain 版本和依赖包。
---

# 临时运行代码片段

在 Bash 或 Zsh 中使用 quoted heredoc，将代码原样交给 stdin：

```bash
run-code TOOLCHAIN[@VERSION] [--package SPEC ...] [--commonjs] [--clean] [--quiet] <<'LANG'
CODE
LANG
```

Fish 不支持 heredoc；用 `printf` 将每行代码送入 stdin：

```fish
printf '%s\n' \
    'const value: string = await Promise.resolve("hello");' \
    'console.log(value);' |
    run-code node@20
```

在 Windows PowerShell 7 中使用单引号 here-string：

```powershell
@'
CODE
'@ | run-code TOOLCHAIN[@VERSION] [--package SPEC ...] [--commonjs] [--clean] [--quiet]
```

使用 `python`、`node`、`rust`、`go` 或 `dotnet`；`javascript` 和 `typescript` 是 `node` 的别名，`csharp` 和 `cs` 是 `dotnet` 的别名。省略版本时使用内置的最新稳定版本。Python 依赖版本使用 `NAME==VERSION`；Rust 依赖可用 `NAME[@VERSION][FEATURE,...]` 指定 Cargo features；.NET 依赖使用 NuGet 的 `NAME[@VERSION]`。Node 默认以 ESM 统一运行 JavaScript/TypeScript，仅在代码明确依赖 CommonJS 时添加 `--commonjs`。C# 使用 .NET 10+ file-based app。Python 和 Node 未指定 `--package` 时直接执行且不创建项目；指定依赖时才创建隔离项目。需要项目时默认保留；仅在明确只需一次结果时添加 `--clean`，仅需代码本身的输出时添加 `--quiet`。直接调用工具，不自行创建项目或检查运行环境。

## Examples

```bash
run-code node@20 --package zod@4 --clean <<'TS'
import { z } from "zod";
console.log(z.object({ id: z.number() }).parse({ id: 1 }));
TS

run-code python@3.14 --package httpx --clean <<'PY'
import httpx
print(httpx.URL("https://example.com").host)
PY

run-code rust@stable --package 'tokio@1[full]' --clean <<'RS'
#[tokio::main]
async fn main() { println!("{}", async { "done" }.await); }
RS

run-code go@latest --package github.com/google/uuid@v1.6.0 --clean <<'GO'
package main
import ("fmt"; "github.com/google/uuid")
func main() { fmt.Println(uuid.New()) }
GO

run-code dotnet@10 --package Spectre.Console --clean <<'CS'
using Spectre.Console;
AnsiConsole.MarkupLine("[green]done[/]");
CS
```

## Package find

找到候选包后，先读取其官方文档、README 和源码入口，确认包名、导入方式、版本与所需 features，再传给 `--package`：

- Python：在 [PyPI](https://pypi.org/search/) 搜索；项目页会链接文档和源码。使用 distribution 名，例如 `--package httpx@0.28.1`。
- JavaScript/TypeScript：用 `npm search KEYWORD` 搜索，用 `npm view PACKAGE repository homepage` 和 `npm view PACKAGE readme` 定位源码及用法。使用 npm spec，例如 `--package zod@4`。
- Rust：用 `cargo search KEYWORD` 搜索，用 `cargo info CRATE` 查看版本、仓库和 features，再阅读 docs.rs。需要 feature 时使用例如 `--package 'tokio@1[full]'`。
- Go：在 [pkg.go.dev](https://pkg.go.dev/) 搜索并阅读包文档和源码，用 `go list -m -versions MODULE` 查看版本。传 module spec，例如 `--package github.com/google/uuid@v1.6.0`，代码中导入实际 package path。
- C#/.NET：在 [NuGet Gallery](https://www.nuget.org/packages) 搜索并阅读 README、项目网站和源码链接；也可用 `dotnet package search KEYWORD`。使用 package ID，例如 `--package Spectre.Console@0.50.0`。
