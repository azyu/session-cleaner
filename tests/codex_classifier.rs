use session_cleaner::{
    classify_session, Classification, Confidence, Decision, Provider, Reason, SessionSample,
};

fn sample(lines: &[&str]) -> SessionSample {
    SessionSample::from_lines(lines.iter().map(|line| (*line).to_owned()).collect())
}

#[test]
fn codex_bootstrap_only_session_is_candidate() {
    let sample = sample(&[
        r#"{"type":"session_meta","payload":{"id":"abc"}}"#,
        r#"{"type":"event_msg","payload":{"type":"thread_name_updated"}}"#,
        r#"{"type":"event_msg","payload":{"type":"task_started"}}"#,
    ]);

    let classification = classify_session(Provider::Codex, &sample);

    assert_eq!(
        classification,
        Classification {
            decision: Decision::Candidate,
            reason: Reason::CodexBootstrapOnly,
            confidence: Confidence::High,
        }
    );
}

#[test]
fn codex_turn_aborted_without_real_content_is_candidate() {
    let sample = sample(&[
        r#"{"type":"session_meta","payload":{"id":"abc"}}"#,
        r#"{"type":"event_msg","payload":{"type":"task_started"}}"#,
        r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<turn_aborted>\nThe user interrupted the previous turn on purpose.\n</turn_aborted>"}]}}"#,
        r#"{"type":"event_msg","payload":{"type":"turn_aborted","reason":"interrupted"}}"#,
    ]);

    let classification = classify_session(Provider::Codex, &sample);

    assert_eq!(classification.decision, Decision::Candidate);
    assert_eq!(classification.reason, Reason::CodexAbortedOnly);
}

#[test]
fn codex_user_content_is_kept() {
    let sample = sample(&[
        r#"{"type":"session_meta","payload":{"id":"abc"}}"#,
        r#"{"type":"message","payload":{"message":{"role":"user","content":"do work"}}}"#,
    ]);

    let classification = classify_session(Provider::Codex, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::ContainsUserContent);
}

#[test]
fn codex_unknown_short_event_is_kept() {
    let sample = sample(&[r#"{"type":"mystery_event","payload":{}}"#]);

    let classification = classify_session(Provider::Codex, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::UnknownEventShape);
}

#[test]
fn parse_failure_is_kept() {
    let sample = sample(&["not json"]);

    let classification = classify_session(Provider::Codex, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::ParseFailure);
}

#[test]
fn omp_is_kept_by_default() {
    let sample = sample(&[r#"{"type":"session","id":"abc"}"#]);

    let classification = classify_session(Provider::Omp, &sample);

    assert_eq!(classification.decision, Decision::Keep);
    assert_eq!(classification.reason, Reason::ProviderReportOnly);
}
