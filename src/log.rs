use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLogEntry {
    pub ts: DateTime<Utc>,
    pub machine: String,
    pub command: String,
    pub outcome: String,
    pub gen_before: Option<u32>,
    pub gen_after: Option<u32>,
    pub duration_ms: u64,
}

pub fn append(entry: &OperationLogEntry) -> Result<()> {
    let path = log_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("couldn't create {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("couldn't open {}", path.display()))?;

    let json = serde_json::to_string(entry).context("couldn't encode log entry")?;
    writeln!(file, "{json}").context("couldn't append log entry")?;
    Ok(())
}

pub fn read_last(limit: usize) -> Result<Vec<OperationLogEntry>> {
    let path = log_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .with_context(|| format!("couldn't open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = vec![];

    for line in reader.lines() {
        let line = line.context("couldn't read log line")?;
        let parsed: OperationLogEntry =
            serde_json::from_str(&line).context("couldn't parse log jsonl entry")?;
        rows.push(parsed);
    }

    if rows.len() <= limit {
        return Ok(rows);
    }

    Ok(rows.split_off(rows.len() - limit))
}

fn log_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("couldn't find home directory")?;
    Ok(home.join(".nina.log"))
}
