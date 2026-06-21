use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn trash_yes_moves_only_candidate_files() {
    let dir = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    let kept = dir.path().join("real-work.jsonl");

    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
"#,
    )
    .unwrap();
    fs::write(
        &kept,
        r#"{"type":"message","payload":{"message":{"role":"user","content":"do work"}}}
"#,
    )
    .unwrap();

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args([
            "trash",
            "--root",
            &format!("codex={}", dir.path().display()),
            "--older-than",
            "0s",
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("moved: 1"))
        .stdout(predicate::str::contains(candidate.display().to_string()));

    assert!(
        !candidate.exists(),
        "candidate should be moved out of place"
    );
    assert!(kept.exists(), "kept file should remain in place");
}

#[test]
fn trash_without_yes_refuses_to_mutate() {
    let dir = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
"#,
    )
    .unwrap();

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args([
            "trash",
            "--root",
            &format!("codex={}", dir.path().display()),
            "--older-than",
            "0s",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires --yes"));

    assert!(candidate.exists(), "candidate should remain without --yes");
}

#[test]
fn trash_yes_writes_mutation_log() {
    let dir = tempdir().unwrap();
    let state = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
"#,
    )
    .unwrap();

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .env("XDG_STATE_HOME", state.path())
        .args([
            "trash",
            "--root",
            &format!("codex={}", dir.path().display()),
            "--older-than",
            "0s",
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("log: "));

    let log_dir = state.path().join("session-cleaner/runs");
    let logs: Vec<_> = fs::read_dir(&log_dir)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(logs.len(), 1);

    let log = fs::read_to_string(logs[0].path()).unwrap();
    let entry: serde_json::Value = serde_json::from_str(log.trim()).unwrap();
    assert_eq!(entry["action"], "trash");
    assert_eq!(entry["result"], "moved");
    assert_eq!(entry["provider"], "codex");
    assert_eq!(entry["reason"], "codex_bootstrap_only");
    assert_eq!(entry["path"], candidate.display().to_string());
}
