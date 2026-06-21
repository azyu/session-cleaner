use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn root_help_shows_quick_start_and_safety_notes() {
    Command::cargo_bin("session-cleaner")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("session-cleaner 0.1.0"))
        .stdout(predicate::str::contains("Quick start:"))
        .stdout(predicate::str::contains(
            "session-cleaner scan --older-than 24h",
        ))
        .stdout(predicate::str::contains("Safety:"))
        .stdout(predicate::str::contains(
            "trash moves files to platform Trash",
        ));
}

#[test]
fn version_flag_prints_package_version() {
    Command::cargo_bin("session-cleaner")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("session-cleaner 0.1.0"));
}

#[test]
fn subcommand_help_documents_flags_and_examples() {
    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args(["scan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("provider=path"))
        .stdout(predicate::str::contains("24h, 7d, 30m"))
        .stdout(predicate::str::contains("-r, --root"))
        .stdout(predicate::str::contains("-c, --config"));

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args(["trash", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "Move high-confidence candidates to platform Trash",
        ))
        .stdout(predicate::str::contains("-y, --yes"));

    Command::cargo_bin("session-cleaner")
        .unwrap()
        .args(["explain", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("--provider claude"));
}

#[test]
fn trash_accepts_short_yes_and_scan_accepts_short_root() {
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
            "-r",
            &format!("codex={}", dir.path().display()),
            "--older-than",
            "0s",
            "-y",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("moved: 1"));

    assert!(!candidate.exists(), "candidate should be moved with -y");
}
