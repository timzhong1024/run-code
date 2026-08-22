# @timzhong2000/run-code

Prebuilt npm distribution of [`run-code`](https://github.com/timzhong1024/run-code).

```bash
echo 'print("hello")' | npx @timzhong2000/run-code python
```

Or copy a source snippet into an isolated template project and pass arguments:

```bash
npx @timzhong2000/run-code node@20 snippet.ts -- first --verbose
```

Set the snippet's working directory and load dotenv variables without changing where the isolated template and dependencies are prepared:

```bash
npx @timzhong2000/run-code node@20 --cwd ./fixtures --env-file ./snippet.env snippet.ts
```

The package selects the bundled binary for the current operating system and CPU architecture.
