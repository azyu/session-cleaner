# Steering

## Current priority

Review and approve the design spec, then scaffold the Rust CLI project.

## Execution mode

- Design first, implementation second.
- Keep the first implementation conservative and reversible.
- Built-in defaults remain primary; optional config exists for overriding roots and age threshold.

## Non-negotiable constraints

- Default behavior must be dry-run scanning.
- Real mutation should move files to Trash before permanent delete is considered.
- Recent files must be protected by an age threshold; initial default is `24h`.
- Ambiguous, unparsable, unsupported, or low-confidence files must be kept.
- OMP cleanup should be report-only in the first implementation unless a high-confidence trivial pattern is established.
- Tests must use fixtures/temp directories and must not touch real `~/.codex`, `~/.claude`, or `~/.omp` paths.
- Claude Code scan coverage must include root and nested JSONL files under `~/.claude/projects/**/`; cleanup rules can stay conservative for nested workflow journals.
- Codex default scan coverage should include both `~/.codex/sessions` and `~/.codex/archived_sessions`.

## Target seams/interfaces

The key seam is provider classification:

```rust
trait Classifier {
    fn provider(&self) -> Provider;
    fn classify(&self, input: &SessionSample) -> Classification;
}
```

External module surface should stay small:

```rust
fn scan(config: &Config) -> Result<ScanReport>;
fn apply(plan: &CleanupPlan, action: Action) -> Result<ApplyReport>;
```

## Decision log

### 2026-06-21 — Use semantic classification, not line count only

Line count catches some Codex bootstrap-only sessions, but misses or misclassifies Claude/OMP patterns. Provider-specific classifiers preserve safety and maintainability.

### 2026-06-21 — Prefer Rust for the recurring CLI

Rust is appropriate for a long-lived local maintenance binary with recursive scanning, streaming JSONL inspection, typed destructive actions, and scheduler-friendly distribution.

### 2026-06-21 — Add optional config loading

The CLI supports `--config` for TOML root and `older_than` overrides after built-in scan/trash behavior was proven. Built-in defaults remain available without a config file.

### 2026-06-21 — Scan nested Claude Code session files

Claude Code stores root session files plus nested subagent/workflow JSONL files under `~/.claude/projects/**/`. The scanner should recurse through that tree, while classifiers keep workflow journals or unclear nested files unless a safe pattern is known.

### 2026-06-21 — Include Codex archived sessions

Codex active sessions under `~/.codex/sessions` are date-partitioned as `YYYY/MM/DD/*.jsonl`; archived sessions also exist under `~/.codex/archived_sessions/*.jsonl`. Both roots should be scanned by default.

### 2026-06-21 — Expand only high-confidence cleanup rules

Safe added patterns: Codex turn-aborted sessions where the only response item is a user `<turn_aborted>` marker, and Claude exit-only sessions with `/exit` plus completion stdout. OMP remains report-only because sampled files all contained assistant/tool activity.

### 2026-06-21 — Confirm OMP session roots from upstream

Upstream `can1357/oh-my-pi` exposes `getSessionsDir()` as `~/.omp/agent/sessions` by default and `getTerminalSessionsDir()` as `~/.omp/agent/terminal-sessions`; XDG mode flattens agent data/state to `$XDG_DATA_HOME/omp/sessions` and `$XDG_STATE_HOME/omp/terminal-sessions`.

### 2026-06-21 — Keep CLI UX boring and explicit

Use standard confirmation wording and aliases: `--yes`/`-y` for non-interactive approval, `--root`/`-r`, and `--config`/`-c`. Help text should foreground safe workflow: scan, explain, then Trash.
