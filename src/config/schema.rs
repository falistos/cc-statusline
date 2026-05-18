//! TOML schema for cc-statusline configuration.
//!
//! Layout follows starship-style sections: one `[module]` table per module,
//! with a top-level `format` string that defines ordering and literals.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub format: String,
    pub model: ModelConfig,
    pub workspace: WorkspaceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: "$workspace $model".to_string(),
            model: ModelConfig::default(),
            workspace: WorkspaceConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    pub aliases: HashMap<String, String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format: "[$name]($style)".to_string(),
            style: "cyan".to_string(),
            aliases: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    /// Number of trailing path segments to keep. 0 = no truncation.
    pub truncate: usize,
    pub truncate_symbol: String,
    /// Style applied when current_dir differs from project_dir (e.g. subdir, worktree).
    pub style_subdir: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format: "[$path]($style)".to_string(),
            style: "blue bold".to_string(),
            truncate: 3,
            truncate_symbol: "…/".to_string(),
            style_subdir: String::new(),
        }
    }
}
