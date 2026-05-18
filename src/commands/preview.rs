//! `preview` — render the statusline with a mock payload.

use crate::config::Format;
use crate::context::Context;
use crate::input::ClaudeInput;
use crate::modules::Registry;
use crate::render;
use anyhow::Result;

const MOCK: &str = r#"{
  "session_id": "preview",
  "transcript_path": "/dev/null",
  "cwd": "/home/dev/projects/cc-statusline",
  "model": { "id": "claude-opus-4-7", "display_name": "Opus 4.7" },
  "workspace": {
    "current_dir": "/home/dev/projects/cc-statusline",
    "project_dir": "/home/dev/projects/cc-statusline"
  },
  "version": "1.0.42",
  "output_style": { "name": "Explanatory" },
  "cost": {
    "total_cost_usd": 0.42,
    "total_duration_ms": 320000,
    "total_lines_added": 120,
    "total_lines_removed": 30
  },
  "context_window": {
    "used_percentage": 65.4,
    "context_window_size": 200000
  },
  "rate_limits": {
    "five_hour": { "used_percentage": 35.2 },
    "seven_day": { "used_percentage": 12.1 }
  }
}"#;

pub fn run() -> Result<()> {
    let input: ClaudeInput = serde_json::from_str(MOCK)?;
    let cfg = crate::config::load().unwrap_or_default();
    let format = Format::parse(&cfg.format)?;
    let ctx = Context::new(input);
    let registry = Registry::new();
    let rendered = render::render_global(&format, &ctx, &cfg, &registry);
    println!("{rendered}");
    Ok(())
}
