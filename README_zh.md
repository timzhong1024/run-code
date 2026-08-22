# run-code

[English](README.md)

在隔离的临时环境中，用指定版本的 runtime/toolchain 和依赖快速运行来自 stdin 或源文件的一段 Python、TypeScript、JavaScript、Rust、Go 或 C# 代码。

## 安装

macOS 推荐使用 Homebrew：

```bash
brew install timzhong1024/tap/run-code
```

也可以通过 npm 安装预编译 binary：

```bash
npm install --global @timzhong2000/run-code
```

或者从源码安装：

```bash
cargo install --locked --git https://github.com/timzhong1024/run-code
```

GitHub Release 还会提供 macOS、Linux 和 Windows 的独立 binary。

`run-code` 按语言调用外部工具；只需安装自己会使用的后端：

| 语言 | 必需工具 |
| --- | --- |
| Python | [uv](https://docs.astral.sh/uv/getting-started/installation/) |
| TypeScript / JavaScript | [Vite+ (`vp`)](https://viteplus.dev/guide/) |
| Rust | [rustup](https://rustup.rs/) |
| Go | [mise](https://mise.jdx.dev/getting-started.html) |
| C# / .NET | [mise](https://mise.jdx.dev/getting-started.html) |

## 示例

### 源文件

```bash
run-code node@20 snippet.ts -- first --verbose
```

执行前，`run-code` 会读取源文件，并将内容复制到新建的隔离模板项目中。它不会在源文件所属的已有工程里运行，不会读取该工程的依赖，也不会复制同目录的其他文件。代码片段需要的依赖应通过 `--package` 明确添加；`--` 后的参数会传给代码进程。

### 工作目录与环境变量

```bash
run-code node@20 --cwd ./fixtures --env-file ./snippet.env snippet.ts
```

`--cwd` 设置最终代码进程看到的工作目录；模板初始化和依赖安装仍然在隔离的临时项目中完成。`--env-file` 按 dotenv 语法为最终启动命令和代码进程加载变量，不修改当前 shell，也不会用于之前的初始化及依赖安装步骤；加载的值不会显示在输出的命令中。两个路径都基于调用 `run-code` 时所在的目录解析。

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

异步 Rust 可以为依赖指定 Cargo features：

```bash
run-code rust@stable --package 'tokio@1[full]' --clean <<'RS'
#[tokio::main]
async fn main() {
    println!("async");
}
RS
```

Windows PowerShell 7 使用单引号 here-string：

```powershell
@'
print("hello\nworld")
'@ | run-code python@3.14 --clean
```

Fish 不支持 heredoc，使用 `printf`：

```fish
printf '%s\n' \
    'const value: string = await Promise.resolve("hello");' \
    'console.log(value);' |
    run-code node@20
```

## Agent Skill

`run-code skill` 从已安装的二进制中输出完整 `SKILL.md`。Agent 会先读取 skill 的名称和 description，在匹配任务后再读取完整内容。

推荐安装到当前项目：

```bash
mkdir -p .agents/skills/run-code-snippet
run-code skill > .agents/skills/run-code-snippet/SKILL.md
```

需要在所有项目中使用时，安装到用户目录：

```bash
mkdir -p ~/.agents/skills/run-code-snippet
run-code skill > ~/.agents/skills/run-code-snippet/SKILL.md
```

Codex 会自动发现这些目录中的 skill；详细约定见 [Codex Skills 文档](https://learn.chatgpt.com/docs/build-skills)。

## 为什么做这个项目

临时运行代码时，初始化项目、安装依赖和准备运行环境的成本很高；临时切换 runtime 或 toolchain 版本也很麻烦。直接安装依赖又容易污染全局环境或当前项目环境。

已经有一些相似工具，但没有同时满足临时依赖、版本切换和隔离运行这些需求。`run-code` 受到这些代码片段运行器、版本管理器和临时包执行工具的启发，把这几个步骤统一成一个命令。

## 安全

`run-code` 提供的是环境和依赖隔离，不是安全沙箱。输入的代码和第三方依赖都以当前用户权限运行，可以访问本机文件、网络、环境变量和凭据。`--env-file` 加载的变量会明确提供给代码片段，不要把密钥传给不可信代码。

依赖安装还可能执行 npm lifecycle scripts、Python build backend、Cargo `build.rs` 或其他生态的构建代码。只运行可信代码和依赖；使用陌生包前先检查官方文档与源码，敏感环境中固定版本，并避免暴露不必要的密钥。`--clean` 只删除临时项目，不会撤销代码已经产生的系统或网络副作用；各包管理器的下载缓存会继续保留。

漏洞请通过 GitHub private vulnerability reporting 私下提交；范围和报告方式见 [SECURITY.md](SECURITY.md)。

## 参数

```text
run-code [OPTIONS] TOOLCHAIN[@VERSION]
run-code [OPTIONS] TOOLCHAIN[@VERSION] FILE [-- ARG ...]
run-code skill
```

- `TOOLCHAIN[@VERSION]`：选择语言及版本。支持 `python`、`node`、`rust`、`go` 和 `dotnet`；`javascript`、`typescript` 是 `node` 的别名，`csharp`、`cs` 是 `dotnet` 的别名，版本可以省略。
- `FILE`：读取源文件，并将内容复制进新的隔离模板项目；不会使用文件所在的已有工程或同目录文件。省略时从 stdin 读取代码。
- `ARG`：在 `--` 后提供并传给代码进程；stdin 输入同样可以传参。
- `-p, --package SPEC`：添加临时依赖，可重复使用以安装多个包。依赖格式遵循对应生态；Python 版本使用 `NAME==VERSION`，Node、Rust、Go 和 .NET 使用 `NAME@VERSION`。Rust 还支持 `NAME[@VERSION][FEATURE,...]`，例如 `'tokio@1[full]'`。
- `--cwd DIR`：设置最终代码进程的工作目录；模板初始化和依赖安装仍与该目录隔离。
- `--env-file FILE`：按 dotenv 语法为最终启动命令和代码进程加载变量；同名变量会覆盖继承的环境变量，值不会显示在输出命令中，但 runner 自身用于隔离的变量优先。
- `--commonjs`：让 Node 以 CommonJS 方式运行；默认使用支持顶层 `await` 的 ESM。
- `--clean`：运行结束后删除临时项目；不指定时保留项目，默认输出的执行命令中会包含项目路径。
- `--quiet`：隐藏项目初始化、依赖安装及命令本身，只输出最终代码进程的 stdout/stderr。
- `skill`：输出内置的 `run-code-snippet` Skill 内容。
- `-h, --help`：显示帮助。
- `-V, --version`：显示版本。

未指定版本时使用内置默认值：Python 3.14、Node latest、Rust stable、Go latest、.NET 10。C# 通过 .NET 10+ file-based app 运行。使用 stdin 且未指定 `--package` 时，Python 和 Node 直接执行；使用文件输入时，即使没有依赖也始终创建隔离模板项目。包管理器的下载缓存保持启用。默认只显示依赖安装和最终代码运行命令，并实时透传它们的 stdout/stderr；项目初始化仅在失败时输出诊断信息。
