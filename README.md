# session-cleaner

[English](README.md) | [한국어](README.ko-kr.md)

> A conservative local CLI for finding low-value Codex, Claude Code, and OMP session files and moving safe candidates to platform Trash.

## Features

- Scans Codex, Claude Code, and OMP session JSONL stores
- Uses provider-specific semantic classifiers instead of line-count-only rules
- Dry-run scanning by default
- Explains why a single session file is kept or selected as a cleanup candidate
- Moves high-confidence candidates to platform Trash; no permanent delete command yet
- Writes mutation logs under `${XDG_STATE_HOME:-~/.local/state}/session-cleaner/runs/`
- Supports text output for humans and JSON output for automation
- Optional TOML config for custom roots and age threshold

## Installation

### From source

Requires Rust 1.94+.

```bash
cargo build --release
cp target/release/session-cleaner ~/.local/bin/session-cleaner
```

Verify the installed binary:

```bash
session-cleaner --version
session-cleaner --help
```

## Quick Start

### 1. Scan without mutating files

```bash
session-cleaner scan --older-than 24h
```

`scan` is safe by default. It reports candidates and does not move or delete files.

### 2. Inspect one file before cleanup

```bash
session-cleaner explain /path/to/session.jsonl
```

Use `--provider` when the path is not under an obvious default root:

```bash
session-cleaner explain /path/to/session.jsonl --provider claude
```

### 3. Move high-confidence candidates to Trash

```bash
session-cleaner trash --older-than 24h --yes
```

> [!CAUTION]
> `trash` moves files to platform Trash. It is reversible on normal desktop setups, but it still mutates your filesystem. Run `scan` first.

## Command Overview

| Command | Purpose |
|---------|---------|
| `session-cleaner scan` | List cleanup candidates. Dry-run by default. |
| `session-cleaner scan --json` | Print a machine-readable scan report. |
| `session-cleaner explain <path>` | Explain the classification for one session file. |
| `session-cleaner trash --yes` | Move high-confidence candidates to platform Trash. |
| `session-cleaner --version` | Print the package version. |

## Default Roots

| Provider | Default roots |
|----------|---------------|
| Codex | `~/.codex/sessions`, `~/.codex/archived_sessions` |
| Claude Code | `~/.claude/projects` |
| OMP | `~/.omp/agent/sessions` |

Claude Code scanning is recursive so nested subagent and workflow JSONL files are inspected. OMP is report-only by default until a high-confidence trivial pattern is established.

## Custom Roots

Use `provider=path` entries with `--root` or `-r`:

```bash
session-cleaner scan -r codex=/tmp/codex-sessions --older-than 7d
session-cleaner trash -r claude=/tmp/claude-projects --older-than 7d --yes
```

Supported providers:

- `codex`
- `claude`
- `omp`

## Configuration

Pass a TOML config with `--config` or `-c`.

```toml
older_than = "24h"

[roots]
codex = ["~/.codex/sessions", "~/.codex/archived_sessions"]
claude = ["~/.claude/projects"]
omp = ["~/.omp/agent/sessions"]
```

Run with:

```bash
session-cleaner scan --config ./session-cleaner.toml
session-cleaner trash --config ./session-cleaner.toml --yes
```

CLI flags override config values for roots and age threshold.

## Safety Model

- `scan` never mutates files.
- `trash` requires `--yes` or `-y`.
- Recent files are protected by `--older-than`; default is `24h`.
- Ambiguous, unparsable, unsupported, or low-confidence files are kept.
- Cleanup rules are provider-specific and intentionally conservative.
- Permanent deletion is not implemented.

## Classification Examples

High-confidence cleanup candidates include:

- Codex bootstrap-only sessions with metadata and startup events but no useful user/assistant work
- Codex turn-aborted sessions where the only response item is a user `<turn_aborted>` marker
- Claude Code interrupted-only sessions with no assistant/tool work
- Claude Code exit-only sessions with `/exit` plus completion stdout such as `Bye!`

Kept sessions include:

- Files with user work content
- Files with assistant content
- Files with tool calls or tool results
- Claude workflow journals or unclear nested files
- OMP files, unless a future safe pattern is added

## JSON Output

```bash
session-cleaner scan --json --older-than 24h
```

Use JSON output when an agent or script needs stable fields such as provider, decision, reason, confidence, byte size, inspected line count, and event types.

## Development

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

## Project Status

This is a local maintenance tool. The current implementation prioritizes safe, reversible cleanup over aggressive deletion.
