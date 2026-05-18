//! Tiny TTL'd disk cache for expensive lookups (currently just git status).
//!
//! Cache files live under `/tmp/cc-statusline-<user>/` on Linux (tmpfs-backed
//! on most distros) and under the platform's standard cache dir elsewhere.
//! Entries are JSON with `expires_at` (unix seconds) and arbitrary `data`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct Entry<T> {
    expires_at: u64,
    data: T,
}

pub fn cache_dir() -> PathBuf {
    let path = platform_cache_dir();
    let _ = std::fs::create_dir_all(&path);
    path
}

#[cfg(target_os = "linux")]
fn platform_cache_dir() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_else(|_| "default".into());
    PathBuf::from(format!("/tmp/cc-statusline-{user}"))
}

#[cfg(not(target_os = "linux"))]
fn platform_cache_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "cc-statusline")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir().join("cc-statusline"))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn get_or_compute<T, F>(key: &str, ttl_secs: u64, compute: F) -> Option<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Option<T>,
{
    let path = cache_dir().join(format!("{key}.json"));
    let current = now();

    if let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(entry) = serde_json::from_str::<Entry<T>>(&content)
        && entry.expires_at > current
    {
        return Some(entry.data);
    }

    let value = compute()?;
    let entry = Entry {
        expires_at: current + ttl_secs,
        data: &value,
    };
    if let Ok(json) = serde_json::to_string(&entry) {
        let _ = std::fs::write(&path, json);
    }
    Some(value)
}

/// Stable, short cache key derived from any hashable input (e.g. a path).
pub fn hash_key(prefix: &str, h: impl std::hash::Hash) -> String {
    use std::hash::{DefaultHasher, Hasher};
    let mut hasher = DefaultHasher::new();
    h.hash(&mut hasher);
    format!("{prefix}-{:x}", hasher.finish())
}
