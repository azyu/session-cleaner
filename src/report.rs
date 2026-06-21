use anyhow::Result;

use crate::scanner::{ScanItem, ScanReport};

pub fn text_report(report: &ScanReport, dry_run: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!("dry-run: {dry_run}\n"));
    out.push_str(&format!("scanned: {}\n", report.scanned));
    out.push_str(&format!("candidates: {}\n", report.candidates));
    out.push_str(&format!("candidate_bytes: {}\n", report.candidate_bytes));
    for item in &report.items {
        out.push_str(&format!(
            "candidate: {} provider={:?} reason={:?} bytes={}\n",
            item.path.display(),
            item.provider,
            item.reason,
            item.bytes
        ));
    }
    out
}

pub fn json_report(report: &ScanReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

pub fn explain_report(item: &ScanItem) -> String {
    format!(
        "path: {}\nprovider: {:?}\ndecision: {:?}\nreason: {:?}\nconfidence: {:?}\nbytes: {}\nlines_inspected: {}\nevent_types: {}\n",
        item.path.display(),
        item.provider,
        item.decision,
        item.reason,
        item.confidence,
        item.bytes,
        item.lines_inspected,
        item.event_types.join(",")
    )
}
