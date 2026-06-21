use assert_cmd::Command;
use predicates::prelude::*;
use std::{fs, thread, time::Duration};
use tempfile::tempdir;

#[test]
fn scan_dry_run_lists_candidate_paths_but_not_kept_paths_as_candidates() {
    let dir = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    let kept = dir.path().join("real-work.jsonl");

    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
{"type":"event_msg","payload":{"type":"thread_name_updated"}}
"#,
    )
    .unwrap();
    fs::write(
        &kept,
        r#"{"type":"message","payload":{"message":{"role":"user","content":"do work"}}}
"#,
    )
    .unwrap();

    thread::sleep(Duration::from_millis(20));

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args([
            "scan",
            "--root",
            &format!("codex={}", dir.path().display()),
            "--older-than",
            "0s",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run: true"))
        .stdout(predicate::str::contains("candidates: 1"))
        .stdout(predicate::str::contains(candidate.display().to_string()))
        .stdout(predicate::str::contains(kept.display().to_string()).not());
}

#[test]
fn scan_uses_default_roots_when_no_root_is_provided() {
    let home = tempdir().unwrap();
    let codex_root = home.path().join(".codex/sessions");
    fs::create_dir_all(&codex_root).unwrap();
    let candidate = codex_root.join("bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
"#,
    )
    .unwrap();

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .env("HOME", home.path())
        .args(["scan", "--older-than", "0s", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("candidates: 1"))
        .stdout(predicate::str::contains(candidate.display().to_string()));
}

#[test]
fn default_roots_include_codex_archived_sessions() {
    let home = tempdir().unwrap();
    let archived_root = home.path().join(".codex/archived_sessions");
    fs::create_dir_all(&archived_root).unwrap();
    let candidate = archived_root.join("archived-bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"archived"}}
"#,
    )
    .unwrap();

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .env("HOME", home.path())
        .args(["scan", "--older-than", "0s", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("candidates: 1"))
        .stdout(predicate::str::contains(candidate.display().to_string()));
}

#[test]
fn scan_reads_roots_and_age_from_config_file() {
    let dir = tempdir().unwrap();
    let config_dir = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
"#,
    )
    .unwrap();
    let config = config_dir.path().join("config.toml");
    fs::write(
        &config,
        format!(
            "older_than = \"0s\"\n\n[roots]\ncodex = [\"{}\"]\n",
            dir.path().display()
        ),
    )
    .unwrap();

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args(["scan", "--config", config.to_str().unwrap(), "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("candidates: 1"))
        .stdout(predicate::str::contains(candidate.display().to_string()));
}
