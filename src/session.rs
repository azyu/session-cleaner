use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SessionSample {
    pub lines: Vec<String>,
    pub parsed_lines: Vec<ParsedLine>,
}

#[derive(Debug, Clone)]
pub enum ParsedLine {
    Json(Value),
    Invalid,
}

impl SessionSample {
    pub fn from_lines(lines: Vec<String>) -> Self {
        let parsed_lines = lines
            .iter()
            .map(|line| match serde_json::from_str::<Value>(line) {
                Ok(value) => ParsedLine::Json(value),
                Err(_) => ParsedLine::Invalid,
            })
            .collect();

        Self {
            lines,
            parsed_lines,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn event_types(&self) -> Vec<String> {
        self.parsed_lines
            .iter()
            .filter_map(|line| match line {
                ParsedLine::Json(value) => event_type(value),
                ParsedLine::Invalid => Some("invalid_json".to_owned()),
            })
            .collect()
    }
}

fn event_type(value: &Value) -> Option<String> {
    let top_level = value.get("type").and_then(Value::as_str)?;
    if top_level == "event_msg" {
        if let Some(payload_type) = value
            .get("payload")
            .and_then(|payload| payload.get("type"))
            .and_then(Value::as_str)
        {
            return Some(format!("event_msg:{payload_type}"));
        }
    }
    Some(top_level.to_owned())
}
