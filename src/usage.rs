//! Usage windows (5h, 7d, model-scoped, spend, extra credits).
//!
//! Claude Code only fills `rate_limits` in the payload right after an API
//! response, and it never carries the model-scoped window. So windows are
//! merged from three sources, freshest first:
//!
//!   1. the payload itself (age 0)
//!   2. `windows.json`, the last payload values seen, written back here
//!   3. `cachedUsageUtilization` in `~/.claude.json`, the only source for the
//!      model-scoped window (e.g. Fable) and the extra-credit balance
//!
//! Each merged window keeps the age of the source it came from so a module
//! can mark values that are no longer fresh.

use crate::cache;
use crate::input::ClaudeInput;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE_FILE: &str = "windows.json";
/// Rewrite the store at most this often when the values have not changed —
/// a 1s refresh interval would otherwise touch the disk every tick.
const STORE_REFRESH_SECS: u64 = 30;
const SNAPSHOT_KEY: &str = "usage-snapshot";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowKind {
    FiveHour,
    SevenDay,
    /// Weekly window scoped to one model, e.g. Fable.
    Scoped,
    /// Gateway spend limit reported in the payload.
    Spend,
    /// Extra-credit balance from the cached snapshot.
    Credits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub kind: WindowKind,
    /// Human label: "5h", "7d", the model name for a scoped window, …
    pub label: String,
    pub percent: f64,
    pub resets_at: Option<u64>,
    /// Seconds since the value was produced. 0 when it comes from the payload.
    #[serde(default)]
    pub age: u64,
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Merges every source into one window list, freshest source winning per kind.
pub fn windows(
    input: &ClaudeInput,
    use_snapshot: bool,
    snapshot_ttl: u64,
    persist: bool,
) -> Vec<Window> {
    let mut merged: Vec<Window> = Vec::new();

    let from_payload = payload_windows(input);
    if persist && !from_payload.is_empty() {
        store_write(&from_payload);
    }
    push_new(&mut merged, from_payload);

    if let Some((stored_at, stored)) = store_read() {
        let age = now().saturating_sub(stored_at);
        push_new(&mut merged, stored.into_iter().map(|w| Window { age, ..w }));
    }

    if use_snapshot && let Some(snapshot) = snapshot(snapshot_ttl) {
        let age = now().saturating_sub(snapshot.fetched_at);
        push_new(
            &mut merged,
            snapshot.windows.into_iter().map(|w| Window { age, ..w }),
        );
    }

    // A window whose reset time has passed carries a stale percentage.
    merged.retain(|w| w.resets_at.is_none_or(|r| r > now()));
    merged
}

fn push_new(merged: &mut Vec<Window>, incoming: impl IntoIterator<Item = Window>) {
    for w in incoming {
        let known = merged
            .iter()
            .any(|m| m.kind == w.kind && (m.kind != WindowKind::Scoped || m.label == w.label));
        if !known {
            merged.push(w);
        }
    }
}

fn payload_windows(input: &ClaudeInput) -> Vec<Window> {
    let Some(rl) = input.rate_limits.as_ref() else {
        return Vec::new();
    };
    [
        (WindowKind::FiveHour, "5h", rl.five_hour.as_ref()),
        (WindowKind::SevenDay, "7d", rl.seven_day.as_ref()),
        (WindowKind::Spend, "spend", rl.spend_limit.as_ref()),
    ]
    .into_iter()
    .filter_map(|(kind, label, w)| {
        let w = w?;
        Some(Window {
            kind,
            label: label.to_string(),
            percent: w.used_percentage?,
            resets_at: w.resets_at,
            age: 0,
        })
    })
    .collect()
}

#[derive(Serialize, Deserialize)]
struct Store {
    stored_at: u64,
    windows: Vec<Window>,
}

fn store_path() -> PathBuf {
    cache::cache_dir().join(STORE_FILE)
}

fn store_read() -> Option<(u64, Vec<Window>)> {
    let raw = std::fs::read_to_string(store_path()).ok()?;
    let store: Store = serde_json::from_str(&raw).ok()?;
    Some((store.stored_at, store.windows))
}

fn store_write(windows: &[Window]) {
    let now = now();
    if let Some((stored_at, stored)) = store_read()
        && now.saturating_sub(stored_at) < STORE_REFRESH_SECS
        && same_values(&stored, windows)
    {
        return;
    }
    let store = Store {
        stored_at: now,
        windows: windows.to_vec(),
    };
    if let Ok(json) = serde_json::to_string(&store) {
        let _ = std::fs::write(store_path(), json);
    }
}

fn same_values(a: &[Window], b: &[Window]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.kind == y.kind && x.percent == y.percent && x.resets_at == y.resets_at)
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    fetched_at: u64,
    windows: Vec<Window>,
}

/// `~/.claude.json` is a few hundred kilobytes, so the extracted snapshot is
/// cached: at a 1s refresh interval we parse it once per TTL instead of every
/// tick.
fn snapshot(ttl: u64) -> Option<Snapshot> {
    cache::get_or_compute(SNAPSHOT_KEY, ttl, read_claude_json)
}

fn read_claude_json() -> Option<Snapshot> {
    let home = directories::BaseDirs::new()?.home_dir().to_path_buf();
    let raw = std::fs::read_to_string(home.join(".claude.json")).ok()?;
    let parsed: ClaudeConfig = serde_json::from_str(&raw).ok()?;
    let cached = parsed.cached_usage_utilization?;
    let utilization = cached.utilization?;
    let fetched_at = cached.fetched_at_ms.unwrap_or(0) / 1000;

    let mut windows: Vec<Window> = utilization
        .limits
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            let percent = entry.percent?;
            let (kind, label) = match entry.kind.as_deref() {
                Some("session") => (WindowKind::FiveHour, "5h".to_string()),
                Some("weekly_all") => (WindowKind::SevenDay, "7d".to_string()),
                _ => {
                    let name = entry
                        .scope
                        .as_ref()
                        .and_then(|s| s.model.as_ref())
                        .and_then(|m| m.display_name.clone())
                        .or_else(|| entry.scope.as_ref().and_then(|s| s.surface.clone()))?;
                    (WindowKind::Scoped, name)
                }
            };
            Some(Window {
                kind,
                label,
                percent,
                resets_at: entry.resets_at.as_deref().and_then(parse_iso8601_utc),
                age: 0,
            })
        })
        .collect();

    if let Some(extra) = utilization.extra_usage
        && extra.is_enabled.unwrap_or(false)
        && let Some(percent) = extra.utilization.filter(|p| *p > 0.0)
    {
        windows.push(Window {
            kind: WindowKind::Credits,
            label: "credits".to_string(),
            percent,
            resets_at: None,
            age: 0,
        });
    }

    Some(Snapshot {
        fetched_at,
        windows,
    })
}

#[derive(Deserialize)]
struct ClaudeConfig {
    #[serde(rename = "cachedUsageUtilization")]
    cached_usage_utilization: Option<CachedUsage>,
}

#[derive(Deserialize)]
struct CachedUsage {
    #[serde(rename = "fetchedAtMs")]
    fetched_at_ms: Option<u64>,
    utilization: Option<Utilization>,
}

#[derive(Deserialize)]
struct Utilization {
    limits: Option<Vec<LimitEntry>>,
    extra_usage: Option<ExtraUsage>,
}

#[derive(Deserialize)]
struct LimitEntry {
    kind: Option<String>,
    percent: Option<f64>,
    resets_at: Option<String>,
    scope: Option<Scope>,
}

#[derive(Deserialize)]
struct Scope {
    model: Option<ScopeModel>,
    surface: Option<String>,
}

#[derive(Deserialize)]
struct ScopeModel {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct ExtraUsage {
    is_enabled: Option<bool>,
    utilization: Option<f64>,
}

/// Parses `2026-09-03T18:00:00.127472+00:00` to epoch seconds. The snapshot
/// always reports UTC, so the offset is ignored and only the civil datetime
/// is converted — no date crate needed for this one format.
fn parse_iso8601_utc(s: &str) -> Option<u64> {
    let bytes = s.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let num = |range: std::ops::Range<usize>| s.get(range)?.parse::<i64>().ok();
    let (year, month, day) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hour, min, sec) = (num(11..13)?, num(14..16)?, num(17..19)?);

    // Days from civil epoch (Howard Hinnant's algorithm).
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;

    u64::try_from(days * 86_400 + hour * 3_600 + min * 60 + sec).ok()
}
