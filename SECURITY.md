# Security Policy

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting for this repository instead of opening a public issue. Include the affected version, operating system, reproduction steps, impact, and any suggested mitigation. Do not include real credentials or other sensitive data in the report.

## Security boundary

`run-code` isolates temporary projects and their dependencies from the current project; it is not a sandbox. Snippets, package installers, build scripts, and package lifecycle hooks run with the current user's permissions and may access local files, environment variables, credentials, the network, and other processes.

Only run trusted code and dependencies. Pin dependency versions when reproducibility matters, inspect unfamiliar packages before use, and avoid running the tool with elevated privileges or unnecessary secrets in the environment. `--clean` removes the generated temporary project after execution, but it cannot undo filesystem, process, or network side effects and does not remove package-manager caches.

The supported security baseline is the latest published release. Security fixes are not guaranteed to be backported to older versions.
