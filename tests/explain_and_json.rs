use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn explain_prints_decision_reason_and_event_types() {
    let dir = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
{"type":"event_msg","payload":{"type":"thread_name_updated"}}
"#,
    )
    .unwrap();

    let output = Command::cargo_bin("session-cleaner")
        .unwrap()
        .args([
            "explain",
            candidate.to_str().unwrap(),
            "--provider",
            "codex",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("provider: Codex"));
    assert!(stdout.contains("decision: Candidate"));
    assert!(stdout.contains("reason: CodexBootstrapOnly"));
    assert!(stdout.contains("event_types: session_meta,event_msg:thread_name_updated"));
}

#[test]
fn scan_json_emits_candidate_items() {
    let dir = tempdir().unwrap();
    let candidate = dir.path().join("bootstrap.jsonl");
    fs::write(
        &candidate,
        r#"{"type":"session_meta","payload":{"id":"abc"}}
"#,
    )
    .unwrap();

    let output = Command::cargo_bin("session-cleaner")
        .unwrap()
        .args([
            "scan",
            "--json",
            "--root",
            &format!("codex={}", dir.path().display()),
            "--older-than",
            "0s",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["scanned"], 1);
    assert_eq!(json["candidates"], 1);
    assert_eq!(json["items"][0]["path"], candidate.display().to_string());
    assert_eq!(json["items"][0]["decision"], "candidate");
    assert_eq!(json["items"][0]["reason"], "codex_bootstrap_only");
}
