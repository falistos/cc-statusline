//! The JSON payload Claude Code sends to the statusline on stdin.
//!
//! Schema: https://code.claude.com/docs/en/statusline
//!
//! Every field is optional: the payload evolves, and several objects only
//! appear under conditions (a fresh API response, a recent Claude Code, a
//! subscription rather than an API key). Fields no module reads are left out.

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ClaudeInput {
    pub session_id: Option<String>,
    pub session_name: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<Model>,
    pub workspace: Option<Workspace>,
    pub version: Option<String>,
    pub output_style: Option<OutputStyle>,
    pub effort: Option<Effort>,
    pub fast_mode: Option<bool>,
    pub cost: Option<Cost>,
    pub context_window: Option<ContextWindow>,
    pub prompt_cache: Option<PromptCache>,
    pub rate_limits: Option<RateLimits>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Model {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Workspace {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OutputStyle {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Effort {
    pub level: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Cost {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_lines_added: Option<u64>,
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ContextWindow {
    pub used_percentage: Option<f64>,
    pub context_window_size: Option<u64>,
    pub total_input_tokens: Option<u64>,
}

/// Requires Claude Code 2.1.251 or later; older builds omit the object.
#[derive(Debug, Deserialize, Default)]
pub struct PromptCache {
    pub warm: Option<bool>,
    pub caching_observed: Option<bool>,
    pub ttl: Option<String>,
    pub expires_at: Option<u64>,
    pub requests: Option<u64>,
    pub misses: Option<u64>,
    pub hit_ratio: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RateLimits {
    pub five_hour: Option<RateLimitWindow>,
    pub seven_day: Option<RateLimitWindow>,
    pub spend_limit: Option<RateLimitWindow>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RateLimitWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<u64>,
}
