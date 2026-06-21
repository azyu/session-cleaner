# Task Board

## Active phase

| Task | Status | Evidence |
|---|---|---|
| Review design spec | Done | User approved spec on 2026-06-21 |
| Scaffold Rust CLI project | Done | `Cargo.toml`, `src/main.rs`, and `src/lib.rs` exist |
| Implement scan/explain dry-run CLI | Done | `cargo run -- scan --dry-run --older-than 0s` scanned 2844 files and listed 60 candidates |
| Implement trash action | Done | `tests/trash_cli.rs` verifies candidate moves, missing `--yes` refusal, and mutation log creation |
| Add tests/fixtures | Done for current MVP | `cargo test` passed 20 tests |
| Release install | Done | `cargo build --release`; copied to `~/.local/bin/session-cleaner`; installed binary ran `--help` and dry-run scan |
| Polish CLI usability | Done | `tests/cli_ux.rs` covers root help, `--version`, subcommand examples, and `-r/-y` aliases |
| Add README docs | Done | `README.md` and `README.ko-kr.md`; README command examples verified against installed CLI and temp config |
| Publish v0.1.0 release | Done | GitHub release `v0.1.0` published with `session-cleaner-v0.1.0-darwin-arm64.tar.gz` and SHA256 asset |

## Completed work

| Date | Work | Evidence |
|---|---|---|
| 2026-06-21 | Sampled existing Codex, Claude Code, and OMP session stores | Observed counts: Codex 2155 files with 23 files <=3 lines; Claude 400 files with 0 files <=3 lines; OMP 122 files with 0 files <=3 lines |
| 2026-06-21 | Wrote initial design spec | `docs/superpowers/specs/2026-06-21-session-cleaner-design.md` |
| 2026-06-21 | Verified current directory was not a git repository | `git rev-parse --is-inside-work-tree` returned exit code 128 |
| 2026-06-21 | Initialized git repository and Rust CLI scaffold | `git init`; Cargo project files created |
| 2026-06-21 | Implemented dry-run scan, JSON report, and explain command | `cargo test` passed 9 tests; dry-run listed 23 Codex candidates |
| 2026-06-21 | Confirmed Claude Code nested session storage | Found 402 Claude JSONL files: 108 root-level project sessions and 294 nested sessions under `~/.claude/projects/**/` |
| 2026-06-21 | Confirmed Codex session storage shape | Found 2155 active session files under `~/.codex/sessions/YYYY/MM/DD/` and 165 archived session files under `~/.codex/archived_sessions/`; archived sessions had 3 files <=3 lines |
| 2026-06-21 | Implemented `trash --yes` action | `cargo test` passed 12 tests; `trash_cli` covers candidate move and missing `--yes` refusal |
| 2026-06-21 | Added Codex archived sessions to default roots | Dry-run scanned 2844 files and listed 26 candidates, including 3 archived Codex candidates |
| 2026-06-21 | Added mutation run logs | `trash_cli` verifies JSONL log creation under `XDG_STATE_HOME/session-cleaner/runs` |
| 2026-06-21 | Added conservative Claude interrupted-only classifier | `claude_classifier` covers interrupted candidate, assistant keep, and workflow journal keep |
| 2026-06-21 | Added optional TOML config loading | `scan_cli` verifies `--config` roots and `older_than` |
| 2026-06-21 | Added Codex turn-aborted classifier | Sampled `session_meta + task_started + response_item(<turn_aborted>) + turn_aborted`; dry-run found 21 `codex_aborted_only` candidates |
| 2026-06-21 | Added Claude exit-only classifier | Requires `/exit` plus completion stdout (`Bye!` or `Worktree removed`); dry-run found 11 `claude_exit_only` candidates |
| 2026-06-21 | Verified expanded candidate breakdown | `scan --json --older-than 0s`: 60 candidates = 26 `codex_bootstrap_only`, 21 `codex_aborted_only`, 11 `claude_exit_only`, 2 `claude_interrupted_only` |
| 2026-06-21 | Researched OMP session storage from upstream source | `can1357/oh-my-pi`; `getSessionsDir()` resolves `~/.omp/agent/sessions` or XDG data override, `getTerminalSessionsDir()` resolves `~/.omp/agent/terminal-sessions` or XDG state override |
| 2026-06-21 | Built and installed release binary | `cargo test` passed 20 tests; `cargo build --release`; installed binary at `~/.local/bin/session-cleaner` ran `--help` and `scan --dry-run --older-than 0s` |
| 2026-06-21 | Polished CLI usability | Added root quick-start/safety help, subcommand examples, `--version`, and short aliases; `cargo test` passed 24 tests |
| 2026-06-21 | Added English and Korean README docs | Followed `azyu/bb-cli` README structure; verified documented CLI commands, config example, placeholder scan, `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings` |
| 2026-06-21 | Published GitHub release v0.1.0 | Cargo version was already `0.1.0`; built macOS arm64 release archive; pushed tag `v0.1.0`; uploaded archive and `.sha256` to GitHub Releases |

## Next observable work

1. Consider permanent delete only after Trash behavior is proven.
2. Add richer provider cleanup patterns only after more safe examples are confirmed.
