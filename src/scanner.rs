use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::{classify_session, Classification, Decision, Provider, SessionSample};

#[derive(Debug, Clone)]
pub struct RootConfig {
    pub provider: Provider,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub roots: Vec<RootConfig>,
    pub older_than: Duration,
    pub max_inspect_lines: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanItem {
    pub path: PathBuf,
    pub provider: Provider,
    pub decision: Decision,
    pub reason: crate::Reason,
    pub confidence: crate::Confidence,
    pub bytes: u64,
    pub lines_inspected: usize,
    pub event_types: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanReport {
    pub scanned: usize,
    pub candidates: usize,
    pub candidate_bytes: u64,
    pub items: Vec<ScanItem>,
}

pub fn scan_roots(config: &ScanConfig) -> Result<ScanReport> {
    let mut scanned = 0;
    let mut candidate_bytes = 0;
    let mut items = Vec::new();

    for root in &config.roots {
        if !root.path.exists() {
            continue;
        }

        for entry in WalkDir::new(&root.path).follow_links(false) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                continue;
            }
            if !is_old_enough(path, config.older_than)? {
                continue;
            }

            scanned += 1;
            let metadata = fs::metadata(path)?;
            let sample = read_sample(path, config.max_inspect_lines)?;
            let classification = classify_session(root.provider, &sample);
            if classification.decision == Decision::Candidate {
                candidate_bytes += metadata.len();
                items.push(to_item(
                    path,
                    root.provider,
                    classification,
                    metadata.len(),
                    &sample,
                ));
            }
        }
    }

    let candidates = items.len();
    Ok(ScanReport {
        scanned,
        candidates,
        candidate_bytes,
        items,
    })
}

pub fn explain_file(provider: Provider, path: &Path, max_inspect_lines: usize) -> Result<ScanItem> {
    let metadata =
        fs::metadata(path).with_context(|| format!("reading metadata for {}", path.display()))?;
    let sample = read_sample(path, max_inspect_lines)?;
    let classification = classify_session(provider, &sample);
    Ok(to_item(
        path,
        provider,
        classification,
        metadata.len(),
        &sample,
    ))
}

fn to_item(
    path: &Path,
    provider: Provider,
    classification: Classification,
    bytes: u64,
    sample: &SessionSample,
) -> ScanItem {
    ScanItem {
        path: path.to_path_buf(),
        provider,
        decision: classification.decision,
        reason: classification.reason,
        confidence: classification.confidence,
        bytes,
        lines_inspected: sample.lines.len(),
        event_types: sample.event_types(),
    }
}

fn read_sample(path: &Path, max_lines: usize) -> Result<SessionSample> {
    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(SessionSample::from_lines(
        content
            .lines()
            .take(max_lines)
            .map(ToOwned::to_owned)
            .collect(),
    ))
}

fn is_old_enough(path: &Path, older_than: Duration) -> Result<bool> {
    let modified = fs::metadata(path)?.modified()?;
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    Ok(age >= older_than)
}
