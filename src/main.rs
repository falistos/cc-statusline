use std::io::{self, Read};
use std::panic;
use std::process::ExitCode;

mod input;

use input::ClaudeInput;

fn main() -> ExitCode {
    panic::set_hook(Box::new(|_| {
        // Statusline must never crash visibly: a panic prints nothing,
        // Claude Code keeps the previous statusline.
    }));

    let result = panic::catch_unwind(run);
    match result {
        Ok(Ok(())) => ExitCode::SUCCESS,
        _ => ExitCode::SUCCESS, // Always exit 0 to avoid Claude Code error display
    }
}

fn run() -> anyhow::Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let input: ClaudeInput = serde_json::from_str(&buf)?;

    // Minimal skeleton output: workspace basename + model.
    // Real rendering pipeline comes in Phase 2-3.
    let cwd = input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(input.cwd.as_deref())
        .unwrap_or("");
    let dir = std::path::Path::new(cwd)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let model = input
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("");

    print!("{dir} | {model}");
    Ok(())
}
