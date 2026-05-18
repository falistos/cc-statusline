//! All on-disk paths for cc-statusline live under `~/.claude/cc-statusline/`.
//!
//! Layout:
//!   ~/.claude/cc-statusline/
//!     ├── config.toml
//!     ├── cache/
//!     └── errors.log

use std::path::PathBuf;

const SUBDIR: &str = "cc-statusline";

pub fn base_dir() -> Option<PathBuf> {
    let home = directories::BaseDirs::new()?.home_dir().to_path_buf();
    Some(home.join(".claude").join(SUBDIR))
}

pub fn config_file() -> Option<PathBuf> {
    base_dir().map(|d| d.join("config.toml"))
}

pub fn cache_dir() -> Option<PathBuf> {
    base_dir().map(|d| d.join("cache"))
}

pub fn error_log() -> Option<PathBuf> {
    base_dir().map(|d| d.join("errors.log"))
}
