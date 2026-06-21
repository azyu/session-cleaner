use session_cleaner::{classify_session, Decision, Provider, Reason, SessionSample};

fn sample(lines: &[&str]) -> SessionSample {
    SessionSample::from_lines(lines.iter().map(|line| (*line).to_owned()).collect())
}

#[test]
fn claude_user_only_interrupted_session_is_candidate() {
    let sample = sample(&[
        r#"{"type":"user","message":{"role":"user","content":"review this"}}"#,
        r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"<command-message>skill</command-message>"}]}}"#,
        r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#,
    ]);

    let classification = classify_session(Provider::Claude, &sample);

    assert_eq!(classification.decision, Decision::Candidate);
    assert_eq!(classification.reason, Reason::ClaudeInterruptedOnly);
}

#[test]
fn claude_exit_only_session_is_candidate() {
    let sample = sample(&[
        r#"{"type":"last-prompt","sessionId":"abc"}"#,
        r#"{"type":"permission-mode","permissionMode":"plan"}"#,
        r#"{"type":"file-history-snapshot","snapshot":{"trackedFileBackups":{}}}"#,
        r#"{"type":"user","message":{"role":"user","content":"<local-command-caveat>Caveat</local-command-caveat>"},"isMeta":true}"#,
        r#"{"type":"user","message":{"role":"user","content":"<command-name>/exit</command-name>\n<command-message>exit</command-message>"}}"#,
        r#"{"type":"user","message":{"role":"user","content":"<local-command-stdout>Bye!</local-command-stdout>"}}"#,
    ]);

    let classification = classify_session(Provider::Claude, &sample);

    assert_eq!(classification.decision, Decision::Candidate);
    assert_eq!(classification.reason, Reason::ClaudeExitOnly);
}

#[test]
fn claude_exit_command_without_completion_stdout_is_kept() {
    let sample = sample(&[
        r#"{"type":"last-prompt","sessionId":"abc"}"#,
        r#"{"type":"user","message":{"role":"user","content":"<command-name>/exit</command-name>\n<command-message>exit</command-message>"}}"#,
    ]);

    let classification = classify_session(Provider::Claude, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::ProviderReportOnly);
}
#[test]
fn claude_assistant_content_is_kept() {
    let sample = sample(&[
        r#"{"type":"user","message":{"role":"user","content":"review this"}}"#,
        r#"{"type":"assistant","message":{"role":"assistant","content":"result"}}"#,
        r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#,
    ]);

    let classification = classify_session(Provider::Claude, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::ContainsAssistantContent);
}

#[test]
fn claude_workflow_journal_is_kept() {
    let sample = sample(&[
        r#"{"type":"started","agentId":"a1"}"#,
        r#"{"type":"result","agentId":"a1","status":"ok"}"#,
    ]);

    let classification = classify_session(Provider::Claude, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::ClaudeWorkflowJournal);
}
