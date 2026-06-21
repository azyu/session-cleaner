use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::{ScanItem, ScanReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashReport {
    pub moved: usize,
    pub paths: Vec<PathBuf>,
    pub log_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct MutationLogEntry<'a> {
    action: &'static str,
    result: &'static str,
    path: String,
    provider: crate::Provider,
    reason: crate::Reason,
    bytes: u64,
    lines_inspected: usize,
    event_types: &'a [String],
}

pub fn trash_candidates(report: &ScanReport) -> Result<TrashReport> {
    let log_path = new_log_path()?;
    let mut paths = Vec::new();
    let mut log = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&log_path)
        .with_context(|| format!("creating mutation log {}", log_path.display()))?;

    for item in &report.items {
        let metadata = fs::metadata(&item.path)
            .with_context(|| format!("rechecking metadata for {}", item.path.display()))?;
        if metadata.len() != item.bytes {
            bail!("file changed since scan: {}", item.path.display());
        }

        trash::delete(&item.path).with_context(|| format!("trashing {}", item.path.display()))?;
        write_log_entry(&mut log, item)?;
        paths.push(item.path.clone());
    }

    Ok(TrashReport {
        moved: paths.len(),
        paths,
        log_path,
    })
}

fn write_log_entry(log: &mut fs::File, item: &ScanItem) -> Result<()> {
    let entry = MutationLogEntry {
        action: "trash",
        result: "moved",
        path: item.path.display().to_string(),
        provider: item.provider,
        reason: item.reason,
        bytes: item.bytes,
        lines_inspected: item.lines_inspected,
        event_types: &item.event_types,
    };
    serde_json::to_writer(&mut *log, &entry)?;
    writeln!(log)?;
    Ok(())
}

fn new_log_path() -> Result<PathBuf> {
    let root = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".session-cleaner-state"));
    let dir = root.join("session-cleaner/runs");
    fs::create_dir_all(&dir).with_context(|| format!("creating log dir {}", dir.display()))?;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Ok(dir.join(format!("{nanos}.jsonl")))
}
