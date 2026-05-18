//! `init` — write a starter config and patch ~/.claude/settings.json.

use crate::paths;
use anyhow::{Context as _, Result};
use std::path::PathBuf;

const TEMPLATE: &str = r##"# cc-statusline configuration
# Documentation: https://github.com/mediavee/cc-statusline
#
# The default values shown below are commented out. Uncomment and edit
# any section to customise its module. Sections are independent.

# Global format. Each `$module` placeholder calls the module of that name.
# Groups in `[ ... ]` (with or without `(style)`) are hidden when none of
# their inner variables produce output, so leading spaces collapse with
# their module.
#
# format = "$workspace[ $git_branch][ $git_status][ $model][ $context][ $cost][ $rate_limits][ $output_style]"

# Per-module configuration (examples):

# [workspace]
# style = "blue bold"
# truncate = 3              # keep N trailing path segments; 0 disables
# truncate_symbol = "…/"
# style_subdir = "blue"     # used when current_dir != project_dir

# [model]
# style = "cyan"
# [model.aliases]
# "claude-opus-4-7" = "opus-4.7"

# [context]
# source = "auto"           # "stdin" | "transcript" | "auto"
# precision = 0
# Visual alternatives — replace the default `[$percent%]($style)` format with
# any of these. Available variables: $percent $remaining $bar $spark $circle
# $gradient_style.
# format = "[$bar $percent%]($style)"            # smooth bar + label
# format = "[$spark $percent%]($style)"          # 1-char sparkline + label
# format = "[$circle $percent%]($gradient_style)" # pie meter + gradient color
# [[context.thresholds]]
# max = 50
# style = "green"
# [[context.thresholds]]
# max = 80
# style = "yellow"
# [[context.thresholds]]
# max = 100
# style = "red bold"
#
# Smooth RGB gradient (overrides thresholds for $gradient_style only):
# [[context.gradient]]
# at = 0
# color = "#0a4d0a"
# [[context.gradient]]
# at = 50
# color = "#ffaa00"
# [[context.gradient]]
# at = 100
# color = "#ff0000"

# [cost]
# style = "yellow"
# precision = 2
# hide_below = 0.01

# [rate_limits]
# style = "magenta"
# hide_below_percent = 5.0
# Per-window variables (h5_/d7_): $h5 $h5_bar $h5_spark $h5_circle $h5_style
# (and same for d7_).
# format = "[5h$h5_spark]($h5_style)[ 7d$d7_spark]($d7_style)"   # ultra-compact
# format = "[5h $h5_bar $h5%]($h5_style)[ 7d $d7_bar $d7%]($d7_style)"
#
# [[rate_limits.gradient]]
# at = 0
# color = "#3366ff"
# [[rate_limits.gradient]]
# at = 100
# color = "#ff3366"

# [viz]                       # Shared options for $bar rendering
# bar_width = 10
# bar_filled = "█"
# bar_empty = "░"
# bar_partial = true          # sub-cell eighths fill for smoother bars

# [git_status]
# cache_ttl_seconds = 5
# show_counts = false

# [cache_hit]
# disabled = false          # off by default — turn on to display
# scope = "session"         # "last" | "session"

# [transcript_stats]
# disabled = false

# [tool_usage]
# disabled = false
"##;

pub fn run(force: bool) -> Result<()> {
    let cfg_path = config_path();
    write_config(&cfg_path, force)?;
    patch_settings()?;
    println!("Done. Reload Claude Code to see the new statusline.");
    Ok(())
}

fn config_path() -> PathBuf {
    paths::config_file().unwrap_or_else(|| PathBuf::from("./cc-statusline/config.toml"))
}

fn write_config(path: &std::path::Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        println!(
            "Config already present at {} (use --force to overwrite).",
            path.display()
        );
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(path, TEMPLATE).with_context(|| format!("writing {}", path.display()))?;
    println!("Wrote config to {}.", path.display());
    Ok(())
}

fn patch_settings() -> Result<()> {
    let Some(base) = directories::BaseDirs::new() else {
        anyhow::bail!("could not resolve home directory");
    };
    let claude_dir = base.home_dir().join(".claude");
    std::fs::create_dir_all(&claude_dir)?;
    let settings_path = claude_dir.join("settings.json");

    let mut value: serde_json::Value = if settings_path.exists() {
        let backup = claude_dir.join("settings.json.bak");
        std::fs::copy(&settings_path, &backup)
            .with_context(|| format!("backing up to {}", backup.display()))?;
        println!("Backed up existing settings.json to {}.", backup.display());
        let raw = std::fs::read_to_string(&settings_path)?;
        serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let exe = std::env::current_exe()?;
    let command = exe.to_string_lossy().to_string();

    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "statusLine".to_string(),
            serde_json::json!({
                "type": "command",
                "command": command,
                "padding": 0
            }),
        );
    }

    std::fs::write(&settings_path, serde_json::to_string_pretty(&value)? + "\n")?;
    println!("Patched {}.", settings_path.display());
    Ok(())
}
