//! Parsing of the JSON payload Claude Code sends to the statusline on stdin.
//!
//! Schema reference: https://code.claude.com/docs/en/statusline
//!
//! All fields are optional because Claude Code's payload schema evolves and
//! we must remain forward/backward compatible.

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ClaudeInput {
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<Model>,
    pub workspace: Option<Workspace>,
    pub version: Option<String>,
    pub output_style: Option<OutputStyle>,
    pub cost: Option<Cost>,
    pub context_window: Option<ContextWindow>,
    pub rate_limits: Option<RateLimits>,
    pub exceeds_200k_tokens: Option<bool>,
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
pub struct Cost {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_api_duration_ms: Option<u64>,
    pub total_lines_added: Option<u64>,
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ContextWindow {
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
    pub context_window_size: Option<u64>,
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RateLimits {
    pub five_hour: Option<RateLimitWindow>,
    pub seven_day: Option<RateLimitWindow>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RateLimitWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<String>,
}
