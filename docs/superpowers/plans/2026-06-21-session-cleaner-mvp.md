# Session Cleaner MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first Rust CLI that scans known agent session roots, explains cleanup candidates, and exposes dry-run candidate reports without deleting files.

**Architecture:** Keep the scanner and classifier as library code behind small interfaces, with CLI commands as thin adapters. Provider-specific rules live in classifiers; filesystem traversal and report formatting stay separate.

**Tech Stack:** Rust 2021, `clap`, `serde`, `serde_json`, `walkdir`, `anyhow`, `tempfile` for tests.

## Global Constraints

- Default behavior must be dry-run scanning.
- Real mutation should move files to Trash before permanent delete is considered.
- Recent files must be protected by an age threshold; initial default is `24h`.
- Ambiguous, unparsable, unsupported, or low-confidence files must be kept.
- OMP cleanup should be report-only in the first implementation unless a high-confidence trivial pattern is established.
- Tests must use fixtures/temp directories and must not touch real `~/.codex`, `~/.claude`, or `~/.omp` paths.

---

### Task 1: Rust CLI Scaffold and Codex Classification

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/classifier.rs`
- Create: `src/session.rs`
- Test: `tests/codex_classifier.rs`

**Interfaces:**
- Produces: `classify_session(provider: Provider, sample: &SessionSample) -> Classification`
- Produces: `Provider`, `Decision`, `Reason`, `Confidence`, `Classification`, `SessionSample`

- [ ] **Step 1: Write failing Codex classifier tests**

Create `tests/codex_classifier.rs` with tests for bootstrap-only candidate, user-content keep, unknown short-event keep, parse-failure keep, and OMP keep-by-default.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test codex_classifier`
Expected: FAIL because the crate/modules do not exist yet.

- [ ] **Step 3: Write minimal scaffold and classifier implementation**

Create Cargo project files and enough library code to satisfy the tests.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test codex_classifier`
Expected: PASS.

### Task 2: Scanner, Reports, and Dry-Run Candidate CLI

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Create: `src/scanner.rs`
- Create: `src/report.rs`
- Test: `tests/scan_cli.rs`

**Interfaces:**
- Consumes: `classify_session(provider: Provider, sample: &SessionSample) -> Classification`
- Produces: `scan_roots(config: ScanConfig) -> Result<ScanReport>`
- Produces CLI: `session-cleaner scan --root codex=<path> --older-than 0s --dry-run`

- [ ] **Step 1: Write failing CLI scan test**

Create `tests/scan_cli.rs` that builds fixture files in a temp directory, runs the binary with a custom root, and asserts candidate paths appear in dry-run output while kept files do not appear as candidates.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test scan_cli`
Expected: FAIL because scan CLI is not implemented.

- [ ] **Step 3: Implement scanner and report output**

Add root parsing, bounded JSONL reading, age filtering, text report, and explicit `--dry-run` flag. Keep `scan` dry-run even when the flag is omitted.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test scan_cli`
Expected: PASS.

### Task 3: Explain Command and JSON Report

**Files:**
- Modify: `src/main.rs`
- Modify: `src/report.rs`
- Modify: `src/scanner.rs`
- Test: `tests/explain_and_json.rs`

**Interfaces:**
- Consumes: `scan_roots(config: ScanConfig) -> Result<ScanReport>`
- Produces CLI: `session-cleaner explain <path> --provider codex`
- Produces CLI: `session-cleaner scan --json --root codex=<path>`

- [ ] **Step 1: Write failing explain/json tests**

Create tests that assert `explain` prints provider, decision, reason, and inspected event types; assert `scan --json` emits parseable JSON with candidate item paths.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test explain_and_json`
Expected: FAIL because explain and JSON reporting are not implemented.

- [ ] **Step 3: Implement explain and JSON report**

Add report serialization and single-file explanation using the same classifier as scan.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test explain_and_json`
Expected: PASS.

### Task 4: Verification and Context Update

**Files:**
- Modify: `.context/PROJECT.md`
- Modify: `.context/TASKS.md`
- Modify: `.context/STEERING.md` if a durable decision changes

**Interfaces:**
- Consumes: all prior tasks.
- Produces: verified project state.

- [ ] **Step 1: Run focused verification**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 2: Run formatter check**

Run: `cargo fmt --check`
Expected: PASS.

- [ ] **Step 3: Run lints**

Run: `cargo clippy -- -D warnings`
Expected: PASS.

- [ ] **Step 4: Update `.context` status**

Record implemented commands and verification evidence.
