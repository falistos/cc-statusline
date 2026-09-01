//! `init` — write a starter config and patch ~/.claude/settings.json.

use crate::paths;
use anyhow::{Context as _, Result};
use std::path::PathBuf;

const TEMPLATE: &str = r##"# cc-statusline configuration
# Every value below is a default; delete a line to keep the default.
# `cc-statusline modules <name>` lists the variables a module exposes.

# A newline in the format string starts a second row.
# format = "$workspace[  $git_branch][ $git_status][  $model][  $session]\n$context[  $prompt_cache][  $rate_limits][  $cost]"

# [context]
# source = "auto"                  # "stdin" | "transcript" | "auto"
# format = "[🧩 $bar $percent%]($style)[ $tokens/$window](#8a8a8a)"

# [rate_limits]
# hide_below_percent = 0.0
# stale_symbol = "~"               # marks values no longer fresh
# use_cached_snapshot = true       # merge ~/.claude.json (scoped window, credits)

# [prompt_cache]
# warn_seconds = 300
# alert_seconds = 60

# [viz]                            # shared $bar options
# bar_width = 10
# bar_filled = "━"
# bar_empty = "─"

# [cache_hit]                      # off by default, reads the transcript
# disabled = false
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
