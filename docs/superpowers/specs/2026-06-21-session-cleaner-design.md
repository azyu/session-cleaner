# Session Cleaner Design

## Problem

Codex, Claude Code, OMP, and similar agent tools leave JSONL session files on disk. Some files contain only startup metadata, cancellation markers, or no meaningful user/assistant work. These low-value files accumulate and make later session search, summarization, and storage management noisier.

A naive rule such as “delete files with three or fewer lines” is too weak as the sole design:

- Codex has real examples where one or two JSONL rows are only `session_meta` and bootstrap events.
- Claude Code has short but non-trivial files above three lines, including interrupted subagent sessions.
- OMP samples did not show three-line-or-shorter sessions, so an aggressive shared line-count rule would do little there and could misclassify future formats.
- Claude Code stores both root session files and nested subagent/workflow JSONL files under `~/.claude/projects/**/`.
- Codex primary sessions under `~/.codex/sessions` use date-partitioned directories (`YYYY/MM/DD/*.jsonl`), and archived Codex sessions also appear under `~/.codex/archived_sessions/*.jsonl`.

The tool should classify trivial sessions by semantic content, not just line count.

## Goals

- Scan known session roots for trivial or nearly empty session files.
- Explain why each file is or is not a cleanup candidate.
- Move candidates to Trash by default rather than deleting them.
- Support deterministic dry-run reports suitable for periodic automation.
- Keep provider-specific session format knowledge isolated behind small classifier interfaces.
- Preserve ambiguous, unparsable, recent, or unsupported files by default.

## Non-goals

- Summarize valuable sessions.
- Rewrite or compact session files in place.
- Infer project importance from working directory names.
- Delete files by default.
- Treat every JSONL file under a provider directory as safe to clean.
- Provide a daemon in the first version. Periodic execution should initially be handled by `launchd`, cron, or another scheduler.

## Recommended implementation language

Rust is the recommended implementation language for the CLI.

Reasons:

- The deliverable is a long-lived local maintenance binary, not a one-off script.
- Streaming JSONL inspection and recursive directory walking are straightforward and fast.
- A typed `Action` model can make destructive behavior explicit.
- A single compiled binary is easy to run from schedulers.
- Rust tests with temporary directories can cover the risky file-moving behavior without touching real session stores.

Python would be acceptable for a prototype, but it is not the preferred shape for a recurring cleanup utility.

## CLI interface

Initial commands:

```bash
session-cleaner scan [--json] [--older-than 24h]
session-cleaner explain <path>
session-cleaner trash [--older-than 24h] [--yes]
```

Deferred command:

```bash
session-cleaner delete [--older-than 30d] --confirm-delete
```

`scan` is the default safe workflow. `trash` performs a reversible move. `delete` should not be included until the scanner and trash behavior have been used safely for a while.

## Default behavior

- Default command behavior is dry-run scanning.
- Files newer than `older_than` are excluded. Initial default: `24h`.
- Symlinks are not followed.
- JSON parse failures are preserved and reported as unsupported.
- Unknown event shapes are preserved unless a provider classifier explicitly marks them trivial.
- Trash is preferred over permanent deletion.
- OMP cleanup is report-only in the first version unless explicitly enabled later.

## Configuration

Configuration is deferred until after the first scanner and trash workflow are proven. The first version should work from built-in defaults for known roots.

```text
~/.config/session-cleaner/config.toml
```

Example:

```toml
older_than = "24h"
default_action = "trash"

[roots]
codex = ["~/.codex/sessions", "~/.codex/archived_sessions"]
claude = "~/.claude/projects"
omp = "~/.omp/agent/sessions"

[tools.codex]
enabled = true
max_inspect_lines = 20

[tools.claude]
enabled = true
max_inspect_lines = 30

[tools.omp]
enabled = false
max_inspect_lines = 30
```

When configuration is implemented, the CLI should still work without a config file by using built-in defaults for known roots.

## Module design

The external interface should stay small:

```rust
fn scan(config: &Config) -> Result<ScanReport>;
fn apply(plan: &CleanupPlan, action: Action) -> Result<ApplyReport>;
```

Internal modules:

```text
Config
  Loads defaults, config file, CLI overrides, root paths, age thresholds.

Scanner
  Walks configured roots, filters by extension, age, symlink policy, and provider.

SessionReader
  Reads bounded JSONL prefixes and file metadata without loading huge sessions fully.

Classifier
  Provider-specific semantic classification.

Planner
  Converts scan results into a CleanupPlan.

Executor
  Performs DryRun, MoveToTrash, or Delete actions.

Reporter
  Emits text and JSON reports.
```

The key seam is `Classifier`. Provider format knowledge belongs there, not in the scanner or executor.

Suggested classifier interface:

```rust
trait Classifier {
    fn provider(&self) -> Provider;
    fn classify(&self, input: &SessionSample) -> Classification;
}
```

`Classification` should include both the decision and the reason:

```rust
enum Decision {
    Candidate,
    Keep,
    Unsupported,
}

struct Classification {
    decision: Decision,
    reason: Reason,
    confidence: Confidence,
}
```

`confidence` is not for machine learning. It is an explicit safety label such as `High`, `Medium`, or `Low` to prevent low-confidence decisions from being applied automatically.

## Provider rules

### Codex

Candidate when all are true:

- File is JSONL or empty.
- File is older than the configured age threshold.
- The inspected rows contain no user message, assistant message, tool call, tool result, or response content.
- Rows are empty or only known bootstrap events such as:
  - `session_meta`
  - `event_msg` with `thread_name_updated`
  - `event_msg` with `task_started`

Default Codex scan roots should include both `~/.codex/sessions` and `~/.codex/archived_sessions`. The date-partitioned `sessions/YYYY/MM/DD/*.jsonl` shape does not require special handling beyond recursive scanning.

Keep when any are true:

- User or assistant content appears.
- Tool call or tool result appears.
- Unknown event type appears in a short file.
- JSON parsing fails.


The Claude scanner should recurse through `~/.claude/projects/**/` because root sessions, nested `subagents/*.jsonl`, and workflow journals can all appear below that tree. Classification must still distinguish mutable candidates from workflow journals that should be kept.
### Claude Code

Candidate only under conservative rules:

- File is older than the configured age threshold.
- No assistant message appears.
- No tool use or tool result appears.
- No file edit, command result, or subagent result appears.
- The content is limited to startup, prompt injection/skill loading, permission metadata, or explicit interruption/cancellation.

Keep when any are true:

- Assistant content appears.
- Tool result appears.
- Session has evidence of file changes or command execution.
- The file is a workflow journal with unclear semantics.
- The classifier cannot explain the decision with a known reason.

### OMP

First version behavior:

- Scan and report counts.
- Do not mark OMP files as cleanup candidates unless the classifier has a high-confidence trivial pattern.
- Keep OMP disabled for `trash` by default.

Reason: observed OMP files were not short by line count, and its session schema needs more examples before cleanup rules should mutate files.

## Reporting

Text report should be optimized for humans:

```text
Codex
  scanned: 2155
  candidates: 23
  candidate_bytes: 472 KiB
  kept: 2132

Claude
  scanned: 400
  candidates: 4
  candidate_bytes: 112 KiB
  kept: 396

OMP
  scanned: 122
  candidates: 0
  kept: 122
```

JSON report should be stable for automation:

```json
{
  "scanned": 2677,
  "candidates": 27,
  "candidate_bytes": 598016,
  "items": [
    {
      "path": "/Users/example/.codex/sessions/...jsonl",
      "provider": "codex",
      "decision": "candidate",
      "reason": "codex_bootstrap_only",
      "bytes": 22040,
      "lines_inspected": 1
    }
  ]
}
```

`explain <path>` should print the classifier decision, inspected event types, and the exact reason. This is important because cleanup tools need user trust.

## Safety model

Actions:

```rust
enum Action {
    DryRun,
    Trash,
    Delete,
}
```

Rules:

- `DryRun` never mutates files.
- `Trash` moves files through a platform Trash mechanism.
- `Delete` requires an explicit command and confirmation flag.
- Low-confidence classifications cannot be trashed or deleted without a future explicit override.
- Applying a cleanup plan should re-check file metadata before mutation to avoid acting on files changed after scanning.

Run logs:

```text
~/.local/state/session-cleaner/runs/YYYY-MM-DDTHH-MM-SS.jsonl
```

Each mutation log entry should include source path, action, provider, reason, size, mtime, and result.

## Testing strategy

Unit tests:

- Codex bootstrap-only files are candidates.
- Codex files with user or assistant content are kept.
- Codex unknown short events are kept.
- Claude interrupted/no-assistant files matching known patterns are candidates.
- Claude files with assistant content, tool results, or workflow journals are kept.
- OMP is kept by default.
- Parse failures are kept.

Integration tests:

- `scan --json` over fixture directories emits stable counts.
- `trash` over a temporary directory moves only candidate files.
- Files newer than `older_than` are preserved.
- Re-checking metadata prevents acting on changed files.

No tests should touch real `~/.codex`, `~/.claude`, or `~/.omp` paths.

## First implementation milestone

Milestone 1:

- Rust CLI scaffold.
- Built-in default roots.
- `scan`, `explain`, and JSON/text reporting.
- Codex classifier with high-confidence bootstrap-only cleanup candidates.
- Claude classifier with conservative no-assistant/interrupted candidates.
- OMP report-only behavior.
- Tests using fixtures and temporary directories.

Milestone 2:

- `trash --yes` action.
- Mutation run logs.
- Metadata re-check before mutation.

Milestone 3:

- Optional config file.
- Optional scheduler examples for `launchd`.
- Consider permanent delete only after Trash behavior is proven.

## Open decisions before implementation

- Final binary name: `session-cleaner` is descriptive but generic.
- Claude scanning should include both root session files and nested subagent/workflow files under `~/.claude/projects/**/`; mutable cleanup rules may remain narrower than scan coverage.
- Whether OMP should remain permanently report-only until more schema examples are collected.
- Which configuration options are worth adding after the built-in default workflow is proven.
