#!/usr/bin/env node

import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const targets = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "win32-x64": "x86_64-pc-windows-msvc"
};

const platform = `${process.platform}-${process.arch}`;
const target = targets[platform];
if (!target) {
  console.error(`run-code does not provide a binary for ${platform}`);
  process.exit(1);
}

const suffix = process.platform === "win32" ? ".exe" : "";
const binary = fileURLToPath(
  new URL(`../vendor/${target}/run-code${suffix}`, import.meta.url)
);
if (!existsSync(binary)) {
  console.error(`run-code binary is missing for ${platform}`);
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(`failed to start run-code: ${result.error.message}`);
  process.exit(1);
}
if (result.signal) {
  process.kill(process.pid, result.signal);
}
process.exit(result.status ?? 1);
