# @timzhong2000/run-code

Prebuilt npm distribution of [`run-code`](https://github.com/timzhong1024/run-code).

```bash
echo 'print("hello")' | npx @timzhong2000/run-code python
```

Or copy a source snippet into an isolated template project and pass arguments:

```bash
npx @timzhong2000/run-code node@20 snippet.ts -- first --verbose
```

The package selects the bundled binary for the current operating system and CPU architecture.
