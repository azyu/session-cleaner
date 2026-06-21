# Project Context

## Project

`session-cleaner` is a planned local CLI for finding low-value agent session files and moving safe cleanup candidates to Trash.

## Current state

- Repository has been initialized with `git init`.
- Rust CLI scaffold exists with scan/explain/trash dry-run-first functionality.
- Release binary installed at `~/.local/bin/session-cleaner`.
- Latest verification: `cargo test`, `cargo build --release`, installed `session-cleaner --help`, and installed dry-run scan all pass.

## Durable references

- Design spec: `docs/superpowers/specs/2026-06-21-session-cleaner-design.md`

## Implemented capabilities

- Rust CLI scaffold.
- `scan` command with dry-run text output.
- `scan --json` report output.
- `explain <path> --provider <provider>` command.
- `trash --yes` command that moves high-confidence candidates to platform Trash.
- Mutation run logs under `${XDG_STATE_HOME:-~/.local/state}/session-cleaner/runs/`.
- Optional TOML config loading via `--config`.
- CLI usability improvements: version flag, command descriptions, quick-start/safety help text, `-r/-c/-y` aliases.
- Recursive scanning for configured roots, including nested Claude Code sessions under `~/.claude/projects/**/` and Codex archived sessions under `~/.codex/archived_sessions`.
- High-confidence Codex bootstrap-only and turn-aborted classifiers.
- Conservative Claude interrupted-only and exit-only cleanup classifiers.
- OMP report-only keep behavior.
- Release install path: `~/.local/bin/session-cleaner`.

## Not yet implemented

- Permanent delete command.

## Verification commands

Project verification commands:

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```
