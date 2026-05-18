pub mod format;
pub mod schema;
pub mod style;

pub use format::Format;
pub use schema::Config;

use anyhow::{Context as _, Result};
use std::path::PathBuf;

const FILENAME: &str = "config.toml";
const APP: &str = "cc-statusline";

/// Resolves the configuration file path, in priority order:
///   1. `$CC_STATUSLINE_CONFIG` env var
///   2. `$XDG_CONFIG_HOME/cc-statusline/config.toml` (via `directories`)
///   3. `~/.claude/cc-statusline.toml`
pub fn config_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CC_STATUSLINE_CONFIG") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    if let Some(dirs) = directories::ProjectDirs::from("", "", APP) {
        let p = dirs.config_dir().join(FILENAME);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(home) = home_dir() {
        let p = home.join(".claude").join("cc-statusline.toml");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn home_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())
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
