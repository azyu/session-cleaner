use serde_json::Value;

use crate::session::{ParsedLine, SessionSample};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Codex,
    Claude,
    Omp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Candidate,
    Keep,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    EmptyFile,
    CodexBootstrapOnly,
    CodexAbortedOnly,
    ContainsUserContent,
    ContainsAssistantContent,
    ContainsToolActivity,
    UnknownEventShape,
    ParseFailure,
    ProviderReportOnly,
    ClaudeInterruptedOnly,
    ClaudeExitOnly,
    ClaudeWorkflowJournal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Classification {
    pub decision: Decision,
    pub reason: Reason,
    pub confidence: Confidence,
}

pub fn classify_session(provider: Provider, sample: &SessionSample) -> Classification {
    match provider {
        Provider::Codex => classify_codex(sample),
        Provider::Claude => classify_claude(sample),
        Provider::Omp => Classification {
            decision: Decision::Keep,
            reason: Reason::ProviderReportOnly,
            confidence: Confidence::Low,
        },
    }
}

fn classify_codex(sample: &SessionSample) -> Classification {
    if sample.is_empty() {
        return Classification {
            decision: Decision::Candidate,
            reason: Reason::EmptyFile,
            confidence: Confidence::High,
        };
    }

    if is_codex_aborted_only(sample) {
        return Classification {
            decision: Decision::Candidate,
            reason: Reason::CodexAbortedOnly,
            confidence: Confidence::High,
        };
    }

    let mut saw_bootstrap = false;

    for parsed in &sample.parsed_lines {
        let ParsedLine::Json(value) = parsed else {
            return keep(Reason::ParseFailure);
        };

        if has_role(value, "user") {
            return keep(Reason::ContainsUserContent);
        }
        if has_role(value, "assistant") {
            return keep(Reason::ContainsAssistantContent);
        }
        if has_tool_activity(value) {
            return keep(Reason::ContainsToolActivity);
        }

        if is_codex_bootstrap_event(value) {
            saw_bootstrap = true;
        } else {
            return keep(Reason::UnknownEventShape);
        }
    }

    if saw_bootstrap {
        Classification {
            decision: Decision::Candidate,
            reason: Reason::CodexBootstrapOnly,
            confidence: Confidence::High,
        }
    } else {
        keep(Reason::UnknownEventShape)
    }
}

fn classify_claude(sample: &SessionSample) -> Classification {
    if sample.is_empty() {
        return Classification {
            decision: Decision::Candidate,
            reason: Reason::EmptyFile,
            confidence: Confidence::High,
        };
    }

    let mut saw_interruption = false;
    let mut saw_workflow_journal = false;
    let mut saw_exit = false;
    let mut saw_exit_completion = false;

    for parsed in &sample.parsed_lines {
        let ParsedLine::Json(value) = parsed else {
            return keep(Reason::ParseFailure);
        };

        if has_role(value, "assistant") {
            return keep(Reason::ContainsAssistantContent);
        }
        if has_tool_activity(value) {
            return keep(Reason::ContainsToolActivity);
        }
        if is_claude_workflow_journal_event(value) {
            saw_workflow_journal = true;
        }
        if contains_text(value, "[Request interrupted by user]") {
            saw_interruption = true;
        }
        if contains_text(value, "<command-name>/exit</command-name>") {
            saw_exit = true;
        }
        if contains_text(value, "<local-command-stdout>Bye!</local-command-stdout>")
            || contains_text(value, "<local-command-stdout>Worktree removed")
        {
            saw_exit_completion = true;
        }
    }

    if saw_workflow_journal {
        return keep(Reason::ClaudeWorkflowJournal);
    }

    if saw_exit && saw_exit_completion {
        return Classification {
            decision: Decision::Candidate,
            reason: Reason::ClaudeExitOnly,
            confidence: Confidence::High,
        };
    }

    if saw_interruption {
        Classification {
            decision: Decision::Candidate,
            reason: Reason::ClaudeInterruptedOnly,
            confidence: Confidence::High,
        }
    } else {
        Classification {
            decision: Decision::Keep,
            reason: Reason::ProviderReportOnly,
            confidence: Confidence::Low,
        }
    }
}

fn keep(reason: Reason) -> Classification {
    Classification {
        decision: Decision::Keep,
        reason,
        confidence: Confidence::High,
    }
}

fn is_codex_bootstrap_event(value: &Value) -> bool {
    match value.get("type").and_then(Value::as_str) {
        Some("session_meta") => true,
        Some("event_msg") => matches!(
            value
                .get("payload")
                .and_then(|payload| payload.get("type"))
                .and_then(Value::as_str),
            Some("thread_name_updated" | "task_started")
        ),
        _ => false,
    }
}

fn is_codex_aborted_only(sample: &SessionSample) -> bool {
    if sample.event_types()
        != [
            "session_meta",
            "event_msg:task_started",
            "response_item",
            "event_msg:turn_aborted",
        ]
    {
        return false;
    }

    let Some(ParsedLine::Json(response_item)) = sample.parsed_lines.get(2) else {
        return false;
    };

    response_item
        .get("payload")
        .and_then(|payload| payload.get("role"))
        .and_then(Value::as_str)
        == Some("user")
        && contains_text(response_item, "<turn_aborted>")
}

fn has_role(value: &Value, role: &str) -> bool {
    value
        .get("message")
        .and_then(|message| message.get("role"))
        .and_then(Value::as_str)
        == Some(role)
        || value
            .get("payload")
            .and_then(|payload| payload.get("message"))
            .and_then(|message| message.get("role"))
            .and_then(Value::as_str)
            == Some(role)
}

fn is_claude_workflow_journal_event(value: &Value) -> bool {
    matches!(
        value.get("type").and_then(Value::as_str),
        Some("started" | "result")
    )
}

fn contains_text(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text.contains(needle),
        Value::Array(values) => values.iter().any(|value| contains_text(value, needle)),
        Value::Object(map) => map.values().any(|value| contains_text(value, needle)),
        _ => false,
    }
}

fn has_tool_activity(value: &Value) -> bool {
    matches!(
        value.get("type").and_then(Value::as_str),
        Some("tool_call" | "tool_result" | "function_call" | "function_call_output")
    )
}
