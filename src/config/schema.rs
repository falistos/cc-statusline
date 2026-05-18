//! TOML schema for cc-statusline configuration.
//!
//! Layout follows starship-style sections: one `[module]` table per module,
//! with a top-level `format` string that defines ordering and literals.
//!
//! All module configs implement `Default`. The defaults below define the
//! out-of-the-box experience — see `Config::default()` for the default
//! global format.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub format: String,
    pub model: ModelConfig,
    pub workspace: WorkspaceConfig,
    pub context: ContextConfig,
    pub cost: CostConfig,
    pub rate_limits: RateLimitsConfig,
    pub output_style: OutputStyleConfig,
    pub version: VersionConfig,
}

impl Default for Config {
    fn default() -> Self {
        // Each module slot is wrapped in `[ $mod]` so the leading space
        // disappears when the module renders nothing.
        Self {
            format: "$workspace[ $model][ $context][ $cost][ $rate_limits][ $output_style]"
                .to_string(),
            model: ModelConfig::default(),
            workspace: WorkspaceConfig::default(),
            context: ContextConfig::default(),
            cost: CostConfig::default(),
            rate_limits: RateLimitsConfig::default(),
            output_style: OutputStyleConfig::default(),
            version: VersionConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Threshold {
    pub max: f64,
    pub style: String,
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

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    pub precision: usize,
    pub thresholds: Vec<Threshold>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format: "[$percent%]($style)".to_string(),
            style: "green".to_string(),
            precision: 0,
            thresholds: vec![
                Threshold {
                    max: 50.0,
                    style: "green".to_string(),
                },
                Threshold {
                    max: 80.0,
                    style: "yellow".to_string(),
                },
                Threshold {
                    max: 100.0,
                    style: "red bold".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CostConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    pub precision: usize,
    /// Hide the module when the total cost is strictly below this value.
    pub hide_below: f64,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            // `\$` escapes a literal `$` in the format grammar.
            format: "[\\$$value]($style)".to_string(),
            style: "yellow".to_string(),
            precision: 2,
            hide_below: 0.01,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct RateLimitsConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    /// Each window (5h / 7d) is hidden if its usage is strictly below this %.
    pub hide_below_percent: f64,
}

impl Default for RateLimitsConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            // Each window is wrapped so it disappears individually below threshold.
            format: "[5h:$h5%]($style)[ 7d:$d7%]($style)".to_string(),
            style: "magenta".to_string(),
            hide_below_percent: 5.0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct OutputStyleConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    pub hide_if_default: bool,
}

impl Default for OutputStyleConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format: "[$name]($style)".to_string(),
            style: "italic dim".to_string(),
            hide_if_default: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct VersionConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            disabled: true,
            format: "[v$version]($style)".to_string(),
            style: "dim".to_string(),
        }
    }
}
