pub mod format;
pub mod schema;
pub mod style;

pub use format::Format;
pub use schema::Config;

use crate::paths;
use anyhow::{Context as _, Result};
use std::path::PathBuf;

/// Resolves the configuration file path, in priority order:
///   1. `$CC_STATUSLINE_CONFIG` env var
///   2. `~/.claude/cc-statusline/config.toml`
pub fn config_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CC_STATUSLINE_CONFIG") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    let p = paths::config_file()?;
    p.exists().then_some(p)
}

pub fn load() -> Result<Config> {
    let Some(path) = config_path() else {
        return Ok(Config::default());
    };
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading config {}", path.display()))?;
    let expanded = expand_env_vars(&raw);
    let cfg: Config =
        toml::from_str(&expanded).with_context(|| format!("parsing config {}", path.display()))?;
    Ok(cfg)
}

/// Substitutes `${VAR}` sequences with environment variables.
/// Unset vars become empty strings. `$$` escapes a literal `$`.
fn expand_env_vars(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('{') => {
                    chars.next();
                    let mut name = String::new();
                    for c in chars.by_ref() {
                        if c == '}' {
                            break;
                        }
                        name.push(c);
                    }
                    if let Ok(v) = std::env::var(&name) {
                        out.push_str(&v);
                    }
                }
                Some('$') => {
                    chars.next();
                    out.push('$');
                }
                _ => out.push('$'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
