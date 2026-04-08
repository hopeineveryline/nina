#![allow(dead_code)]
use anyhow::Context as _;
use std::fs::{self, OpenOptions};
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
}

pub fn is_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::SeqCst)
}

pub fn log(kind: &str, msg: &str) {
    if !is_enabled() {
        return;
    }
    if let Ok(path) = debug_log_path() {
        let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let line = format!("[{}] [{}] {}\n", ts, kind, msg);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = f.write_all(line.as_bytes());
        }
    }
}

pub fn log_command(cmd: &str, machine: &str) {
    log("command", &format!("{} on {}", cmd, machine));
}

pub fn log_error(context: &str, msg: &str) {
    log("error", &format!("{}: {}", context, msg));
}

pub fn log_state(where_: &str, msg: &str) {
    log("state", &format!("{}: {}", where_, msg));
}

pub fn log_output(line: &str) {
    if line.len() > 500 {
        log(
            "output",
            &format!("{}... ({} chars)", &line[..500], line.len()),
        );
    } else {
        log("output", line);
    }
}

pub fn log_result(where_: &str, ok: bool) {
    log(
        "result",
        &format!("{}: {}", where_, if ok { "ok" } else { "FAILED" }),
    );
}

fn debug_log_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("couldn't find home directory")?;
    let dir = home.join(".nina");
    fs::create_dir_all(&dir).context("couldn't create .nina dir")?;
    Ok(dir.join("debug.log"))
}

pub fn clear_log() {
    if let Ok(path) = debug_log_path() {
        let _ = fs::write(&path, "");
    }
}
