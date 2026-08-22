# run-code

[English](README.md)

使用指定版本的 runtime/toolchain 和临时依赖，运行一段一次性的 Python、TypeScript/JavaScript、Rust、Go 或 C# 代码，同时不修改全局环境或当前项目。

```bash
echo 'print("hello")' | run-code python@3.14
```

## 为什么需要 run-code

一个小实验不应该要求你创建项目、选择包管理布局、切换当前 runtime、安装依赖，最后再删除所有文件。把包直接安装到全局或当前项目虽然开始得快，却会留下与项目无关的状态。

`run-code` 把一次性运行收敛为一个命令：

1. 选择 runtime 或 toolchain 版本；
2. 在隔离的临时环境中准备依赖；
3. 运行来自 stdin 或源文件的代码。

它适合试用 README 里看到的包、验证特定 runtime 的行为、复现小型示例，或者让 Agent 做一次聚焦检查。多文件程序、长期依赖和正式构建配置仍应使用普通项目。`run-code` 提供的是便利性的隔离，不是安全沙箱。

## 给 Agent 使用

安装 binary 后，Agent 可以用下面的命令读取内置的 `run-code-snippet` skill，以及与当前版本准确匹配的说明：

```bash
run-code skill
```

如果只希望当前项目使用，将 skill 安装到项目：

```bash
mkdir -p .agents/skills/run-code-snippet
run-code skill > .agents/skills/run-code-snippet/SKILL.md
```

需要跨项目复用时，安装到全局：

```bash
mkdir -p ~/.agents/skills/run-code-snippet
run-code skill > ~/.agents/skills/run-code-snippet/SKILL.md
```

一次性检查的核心调用方式是：

```bash
run-code TOOLCHAIN[@VERSION] [--package SPEC ...] [--clean] [--quiet] <<'LANG'
CODE
LANG
```

这样 Agent 可以明确指定 runtime、使用一次性依赖和隔离项目状态，并获得正常的 stdout/stderr，无需自行初始化项目。

## 安装

macOS：

```bash
brew install timzhong1024/tap/run-code
```

其他安装方式：

```bash
npm install --global @timzhong2000/run-code
cargo install --locked --git https://github.com/timzhong1024/run-code
```

[GitHub Releases](https://github.com/timzhong1024/run-code/releases) 还提供 macOS、Linux 和 Windows 的独立 binary。

`run-code` 把 runtime 安装交给现有工具；只需安装会用到的后端：

| 代码 | Toolchain 参数 | 必需后端 |
| --- | --- | --- |
| Python | `python[@VERSION]` | [uv](https://docs.astral.sh/uv/getting-started/installation/) |
| TypeScript / JavaScript | `node[@VERSION]` | [Vite+ (`vp`)](https://viteplus.dev/guide/) |
| Rust | `rust[@VERSION]` | [rustup](https://rustup.rs/) |
| Go | `go[@VERSION]` | [mise](https://mise.jdx.dev/getting-started.html) |
| C# | `dotnet[@VERSION]` | [mise](https://mise.jdx.dev/getting-started.html) |

## 快速示例

### 使用 npm 包的 TypeScript

```bash
run-code node@20 --package zod@4 --clean <<'TS'
import { z } from "zod";
console.log(await Promise.resolve(z.object({ id: z.number() }).parse({ id: 1 })));
TS
```

### 使用 PyPI 包的 Python

```bash
run-code python@3.14 --package requests==2.32.5 --clean <<'PY'
import requests
print(requests.__version__)
PY
```

### 使用 crate 的 Rust

```bash
run-code rust@stable --package serde_json@1 --clean <<'RS'
fn main() {
    println!("{}", serde_json::json!({"hello": "world"}));
}
RS
```

### 源文件、参数、工作目录与环境变量

```bash
run-code node@20 \
  --package zod@4 \
  --cwd ./fixtures \
  --env-file ./snippet.env \
  snippet.ts -- first --verbose
```

源文件会被复制到全新的隔离模板中；它原有的项目、依赖和同目录文件都不会被使用。`--` 后的参数会传给代码进程。`--cwd` 只影响最终代码进程，`--env-file` 则提供 dotenv 变量，不修改当前 shell，也不会在展示的命令中暴露变量值。

## 执行行为

- 省略版本时使用内置的稳定策略：Python 3.14、Node latest、Rust stable、Go latest、.NET 10。
- Python 和 Node 从 stdin 读取且没有 `--package` 时直接运行；有依赖或使用源文件时会创建隔离模板项目。
- 临时项目默认保留，方便检查输出命令中的生成路径；只运行一次时添加 `--clean`。
- 包管理器下载缓存保持启用，因此隔离运行不等于每次重新下载所有依赖。
- 默认展示依赖安装和最终运行命令，并透传输出；`--quiet` 只保留最终进程的 stdout/stderr。
- Node 默认使用支持 TypeScript 和顶层 `await` 的 ESM；只有明确依赖 CommonJS 时才使用 `--commonjs`。

## 参数

```text
run-code [OPTIONS] TOOLCHAIN[@VERSION]
run-code [OPTIONS] TOOLCHAIN[@VERSION] FILE [-- ARG ...]
run-code skill
```

| 参数 | 含义 |
| --- | --- |
| `TOOLCHAIN[@VERSION]` | `python`、`node`、`rust`、`go` 或 `dotnet`；`javascript`/`typescript` 是 `node` 别名，`csharp`/`cs` 是 `dotnet` 别名 |
| `FILE` | 不读取 stdin，改为复制并运行一个源文件 |
| `-- ARG ...` | 向代码进程传递参数 |
| `-p, --package SPEC` | 添加依赖；多个依赖可重复使用 |
| `--cwd DIR` | 设置最终代码进程的工作目录 |
| `--env-file FILE` | 为最终进程加载 dotenv 变量 |
| `--commonjs` | 让 Node 使用 CommonJS 而不是 ESM |
| `--clean` | 执行后删除临时项目 |
| `--quiet` | 只显示最终进程 stdout/stderr |
| `skill` | 输出内置的 Agent skill |

依赖格式遵循各自生态。Python 使用 `NAME==VERSION`；Node、Rust、Go 和 .NET 使用 `NAME@VERSION`。Rust features 使用 `NAME[@VERSION][FEATURE,...]`，例如 `'tokio@1[full]'`。

### Shell 输入

Bash 和 Zsh 使用上面展示的 quoted heredoc。Fish 可以使用 `printf`：

```fish
printf '%s\n' \
    'const value: string = await Promise.resolve("hello");' \
    'console.log(value);' |
    run-code node@20
```

PowerShell 7 可以使用单引号 here-string：

```powershell
@'
print("hello\nworld")
'@ | run-code python@3.14 --clean
```

## 安全

代码片段、依赖、包 lifecycle hook、Python build backend 和 Cargo build script 都以当前用户权限运行，可以访问文件、网络、环境变量、凭据和其他进程。只运行可信代码和依赖；需要可复现时固定版本，不要把密钥传给不可信代码。

`--clean` 只能删除临时项目，不能撤销文件系统或网络副作用。完整安全边界和漏洞报告方式见 [SECURITY.md](SECURITY.md)。
