//! Append-only diagnostic log for the statusline binary.
//!
//! Statusline mode must never print to stderr (Claude Code would interleave it
//! with the rendered bar), and any error makes the bar render empty — which is
//! invisible to the user. This module persists those failures to
//! `~/.claude/cc-statusline/errors.log` so they can be diagnosed after the fact.
//!
//! All writes are best-effort: any IO failure is swallowed.

use crate::paths;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 256 * 1024;
const RETAIN_TAIL_BYTES: u64 = 64 * 1024;

pub fn log(kind: &str, details: &str) {
    let Some(path) = paths::error_log() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{ts}] {kind}: {}\n", details.replace('\n', " | "));

    let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    let _ = f.write_all(line.as_bytes());
    drop(f);

    // Cheap tail-truncation: if the file gets too big, keep only the last
    // RETAIN_TAIL_BYTES. Skipped silently on any IO failure.
    if let Ok(meta) = std::fs::metadata(&path)
        && meta.len() > MAX_BYTES
    {
        let _ = truncate_head(&path);
    }
}

fn truncate_head(path: &std::path::Path) -> std::io::Result<()> {
    let mut f = OpenOptions::new().read(true).write(true).open(path)?;
    let len = f.metadata()?.len();
    let start = len.saturating_sub(RETAIN_TAIL_BYTES);
    f.seek(SeekFrom::Start(start))?;
    let mut tail = Vec::with_capacity(RETAIN_TAIL_BYTES as usize);
    f.read_to_end(&mut tail)?;
    // Drop any partial first line so the file stays parseable.
    if let Some(nl) = tail.iter().position(|&b| b == b'\n') {
        tail.drain(..=nl);
    }
    f.set_len(0)?;
    f.seek(SeekFrom::Start(0))?;
    f.write_all(&tail)?;
    Ok(())
}
