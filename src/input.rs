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
    pub session_name: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<Model>,
    pub workspace: Option<Workspace>,
    pub worktree: Option<Worktree>,
    pub version: Option<String>,
    pub output_style: Option<OutputStyle>,
    pub effort: Option<Effort>,
    pub thinking: Option<Thinking>,
    pub fast_mode: Option<bool>,
    pub cost: Option<Cost>,
    pub context_window: Option<ContextWindow>,
    pub prompt_cache: Option<PromptCache>,
    pub rate_limits: Option<RateLimits>,
    pub pr: Option<PullRequest>,
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
    pub git_worktree: Option<String>,
    pub repo: Option<Repo>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Repo {
    pub host: Option<String>,
    pub owner: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Worktree {
    pub name: Option<String>,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub original_branch: Option<String>,
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
pub struct Thinking {
    pub enabled: Option<bool>,
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

/// Prompt cache statistics for the main conversation. Requires Claude Code
/// 2.1.251 or later; older builds simply omit the object.
#[derive(Debug, Deserialize, Default)]
pub struct PromptCache {
    pub warm: Option<bool>,
    pub caching_observed: Option<bool>,
    pub ttl: Option<String>,
    pub expires_at: Option<u64>,
    pub requests: Option<u64>,
    pub misses: Option<u64>,
    pub hit_ratio: Option<f64>,
    pub recache_tokens_if_cold: Option<u64>,
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

#[derive(Debug, Deserialize, Default)]
pub struct PullRequest {
    pub number: Option<u64>,
    pub url: Option<String>,
    pub review_state: Option<String>,
    pub kind: Option<String>,
}
