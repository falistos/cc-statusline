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
    pub viz: VizConfig,
    pub model: ModelConfig,
    pub workspace: WorkspaceConfig,
    pub context: ContextConfig,
    pub cost: CostConfig,
    pub rate_limits: RateLimitsConfig,
    pub output_style: OutputStyleConfig,
    pub version: VersionConfig,
    pub cache_hit: CacheHitConfig,
    pub transcript_stats: TranscriptStatsConfig,
    pub tool_usage: ToolUsageConfig,
    pub git_branch: GitBranchConfig,
    pub git_status: GitStatusConfig,
}

impl Default for Config {
    fn default() -> Self {
        // Each module slot is wrapped in `[ $mod]` so the leading space
        // disappears when the module renders nothing.
        Self {
            format: "$workspace[ $git_branch][ $git_status][ $model][ $context][ $cost][ $rate_limits][ $output_style]"
                .to_string(),
            viz: VizConfig::default(),
            model: ModelConfig::default(),
            workspace: WorkspaceConfig::default(),
            context: ContextConfig::default(),
            cost: CostConfig::default(),
            rate_limits: RateLimitsConfig::default(),
            output_style: OutputStyleConfig::default(),
            version: VersionConfig::default(),
            cache_hit: CacheHitConfig::default(),
            transcript_stats: TranscriptStatsConfig::default(),
            tool_usage: ToolUsageConfig::default(),
            git_branch: GitBranchConfig::default(),
            git_status: GitStatusConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Threshold {
    pub max: f64,
    pub style: String,
}

/// Shared visualization options used by `$bar` rendering across modules.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct VizConfig {
    pub bar_width: usize,
    pub bar_filled: String,
    pub bar_empty: String,
    /// When true, the partially-filled cell uses sub-cell eighths
    /// (`▏▎▍▌▋▊▉█`) for smoother fill. Only honored with the default
    /// `bar_filled = "█"` / `bar_empty = "░"`.
    pub bar_partial: bool,
}

impl Default for VizConfig {
    fn default() -> Self {
        Self {
            bar_width: 10,
            bar_filled: "█".to_string(),
            bar_empty: "░".to_string(),
            bar_partial: true,
        }
    }
}

/// One color stop in a gradient. `at` is the percentage (0..=100) at which
/// the color applies; values between stops are linearly interpolated.
#[derive(Debug, Deserialize, Clone)]
pub struct GradientStop {
    pub at: f64,
    pub color: String,
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

#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContextSource {
    /// Use `context_window.used_percentage` from the stdin payload. Fastest.
    Stdin,
    /// Re-compute from the transcript's last assistant message.
    /// Slightly slower but immune to issue #13783 (cumulative tokens bug).
    Transcript,
    /// Prefer stdin, fall back to transcript if stdin's value looks suspect
    /// (e.g. > 100% or 0% with a non-empty transcript).
    #[default]
    Auto,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    pub precision: usize,
    pub source: ContextSource,
    /// Fallback context window size (tokens) when neither the payload nor the
    /// transcript exposes one. Used to compute the percent in transcript mode.
    pub default_window_size: u64,
    pub thresholds: Vec<Threshold>,
    /// When non-empty, `$gradient_style` resolves to a `#rrggbb` interpolated
    /// across these stops. Otherwise `$gradient_style` falls back to the
    /// threshold-picked style.
    pub gradient: Vec<GradientStop>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format: "[$percent%]($style)".to_string(),
            style: "green".to_string(),
            precision: 0,
            source: ContextSource::Auto,
            default_window_size: 200_000,
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
            gradient: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CacheHitConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    /// Aggregation scope: "last" (last assistant message) or "session"
    /// (cumulative across the whole transcript).
    pub scope: CacheHitScope,
    pub precision: usize,
}

#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheHitScope {
    Last,
    #[default]
    Session,
}

impl Default for CacheHitConfig {
    fn default() -> Self {
        Self {
            disabled: true,
            format: "[cache:$pct%]($style)".to_string(),
            style: "cyan dim".to_string(),
            scope: CacheHitScope::Session,
            precision: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct TranscriptStatsConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
}

impl Default for TranscriptStatsConfig {
    fn default() -> Self {
        Self {
            disabled: true,
            format: "[$messages msgs]($style)".to_string(),
            style: "dim".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ToolUsageConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
}

impl Default for ToolUsageConfig {
    fn default() -> Self {
        Self {
            disabled: true,
            format: "[$count tools]($style)".to_string(),
            style: "dim".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GitBranchConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    /// Truncate branch names longer than this. 0 disables.
    pub truncate: usize,
    pub truncate_symbol: String,
}

impl Default for GitBranchConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format: "[($branch)]($style)".to_string(),
            style: "yellow".to_string(),
            truncate: 0,
            truncate_symbol: "…".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GitStatusConfig {
    pub disabled: bool,
    pub format: String,
    pub style: String,
    pub cache_ttl_seconds: u64,
    /// When true, status markers include their count (e.g. "!3" instead of "!").
    pub show_counts: bool,
    pub modified_symbol: String,
    pub untracked_symbol: String,
    pub added_symbol: String,
    pub deleted_symbol: String,
    pub renamed_symbol: String,
    pub conflicted_symbol: String,
    pub ahead_symbol: String,
    pub behind_symbol: String,
    /// Shown only when the tree is fully clean and up to date.
    pub clean_symbol: String,
}

impl Default for GitStatusConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            format:
                "[$conflicted$modified$deleted$renamed$added$untracked$ahead$behind$clean]($style)"
                    .to_string(),
            style: "red".to_string(),
            cache_ttl_seconds: 5,
            show_counts: false,
            modified_symbol: "!".to_string(),
            untracked_symbol: "?".to_string(),
            added_symbol: "+".to_string(),
            deleted_symbol: "✗".to_string(),
            renamed_symbol: "»".to_string(),
            conflicted_symbol: "≠".to_string(),
            ahead_symbol: "⇡".to_string(),
            behind_symbol: "⇣".to_string(),
            clean_symbol: String::new(),
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
    pub thresholds: Vec<Threshold>,
    /// When non-empty, `$h5_gradient_style` / `$d7_gradient_style` interpolate
    /// across these stops. Otherwise they fall back to the threshold pick.
    pub gradient: Vec<GradientStop>,
}

impl Default for RateLimitsConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            // Each window is wrapped so it disappears individually below threshold.
            format: "[5h:$h5%]($style)[ 7d:$d7%]($style)".to_string(),
            style: "magenta".to_string(),
            hide_below_percent: 5.0,
            thresholds: vec![],
            gradient: vec![],
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
