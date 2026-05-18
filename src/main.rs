use std::io::{self, Read};
use std::panic;
use std::process::ExitCode;

mod config;
mod context;
mod input;
mod modules;
mod render;
mod transcript;

use config::Format;
use context::Context;
use input::ClaudeInput;
use modules::Registry;

fn main() -> ExitCode {
    panic::set_hook(Box::new(|_| {
        // Statusline must never crash visibly: swallow panics so Claude Code
        // keeps the previous statusline rather than rendering an error.
    }));

    let _ = panic::catch_unwind(run);
    // Always exit 0 — non-zero would make Claude Code show an error banner.
    ExitCode::SUCCESS
}

fn run() {
    let mut buf = String::new();
    if io::stdin().read_to_string(&mut buf).is_err() {
        return;
    }
    let input: ClaudeInput = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(_) => return,
    };

    let cfg = config::load().unwrap_or_default();
    let format = Format::parse(&cfg.format).unwrap_or_else(|_| Format(vec![]));
    let ctx = Context::new(input);
    let registry = Registry::new();

    let rendered = render::render_global(&format, &ctx, &cfg, &registry);
    print!("{rendered}");
}
