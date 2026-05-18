//! Transcript (JSONL) parsing.
//!
//! Two reading strategies:
//!
//! - [`read_last_usage`] does a reverse-read in 16 KiB chunks until it finds
//!   the most recent `assistant` line carrying `usage`. Used by the context
//!   percentage module (transcript mode) and by cache_hit. Cheap even on
//!   very long transcripts.
//!
//! - [`read_full_stats`] streams the whole file line by line. Required by
//!   `transcript_stats` and `tool_usage` since they need cumulative counts.

#![allow(dead_code)]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const CHUNK_SIZE: u64 = 16 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct TranscriptUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
}

impl TranscriptUsage {
    /// Tokens that contribute to the *current* context window: the model's
    /// input + everything it read from cache + everything it created in cache.
    pub fn context_tokens(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.cache_creation)
            .saturating_add(self.cache_read)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TranscriptStats {
    pub user_messages: u64,
    pub assistant_messages: u64,
    pub tool_uses: u64,
    pub last_tool_name: Option<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation: u64,
    pub total_cache_read: u64,
}

/// Walks the file backwards in chunks, returning the usage of the last
/// `assistant` line carrying a `usage` block. Tolerant to malformed lines.
pub fn read_last_usage(path: &Path) -> Option<TranscriptUsage> {
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len == 0 {
        return None;
    }

    let mut tail: Vec<u8> = Vec::new();
    let mut end = len;

    while end > 0 {
        let start = end.saturating_sub(CHUNK_SIZE);
        let to_read = (end - start) as usize;
        file.seek(SeekFrom::Start(start)).ok()?;
        let mut buf = vec![0u8; to_read];
        file.read_exact(&mut buf).ok()?;
        // Append the partial first line carried over from the previous (later) chunk.
        buf.extend_from_slice(&tail);
        tail.clear();

        let mut lines: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();
        // Unless we're at the start of the file, the first line may be truncated:
        // save it for the next round and skip it now.
        if start > 0 {
            if let Some(first) = lines.first() {
                tail = first.to_vec();
            }
            if !lines.is_empty() {
                lines.remove(0);
            }
        }

        for line in lines.iter().rev() {
            if line.is_empty() {
                continue;
            }
            if let Some(u) = parse_usage_line(line) {
                return Some(u);
            }
        }

        end = start;
    }
    None
}

/// Streams the whole transcript and aggregates per-message and per-tool stats.
pub fn read_full_stats(path: &Path) -> Option<TranscriptStats> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut stats = TranscriptStats::default();
    for line in reader.lines().map_while(Result::ok) {
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("user") => stats.user_messages += 1,
            Some("assistant") => {
                stats.assistant_messages += 1;
                if let Some(usage) = v.get("message").and_then(|m| m.get("usage")) {
                    stats.total_input_tokens += usage
                        .get("input_tokens")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                    stats.total_output_tokens += usage
                        .get("output_tokens")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                    stats.total_cache_creation += usage
                        .get("cache_creation_input_tokens")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                    stats.total_cache_read += usage
                        .get("cache_read_input_tokens")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                }
                if let Some(content) = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            stats.tool_uses += 1;
                            if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                                stats.last_tool_name = Some(name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Some(stats)
}

fn parse_usage_line(line: &[u8]) -> Option<TranscriptUsage> {
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    if value.get("type").and_then(|t| t.as_str()) != Some("assistant") {
        return None;
    }
    let usage = value.get("message")?.get("usage")?;
    let input = usage.get("input_tokens")?.as_u64()?;
    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cc = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cr = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some(TranscriptUsage {
        input_tokens: input,
        output_tokens: output,
        cache_creation: cc,
        cache_read: cr,
    })
}
