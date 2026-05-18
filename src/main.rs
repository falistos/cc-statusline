use std::io::{self, Read};
use std::panic;
use std::process::ExitCode;

mod cache;
mod cli;
mod commands;
mod config;
mod context;
mod git;
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

    let result = panic::catch_unwind(dispatch);
    match result {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(_)) => {
            // In subcommand mode an error already printed to stderr.
            // In statusline mode we still want to exit 0 to avoid the
            // error banner; dispatch() never returns Err in that mode.
            ExitCode::FAILURE
        }
        Err(_) => ExitCode::SUCCESS,
    }
}

fn dispatch() -> anyhow::Result<()> {
    let cli = match cli::parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("{e}");
            cli::print_help();
            std::process::exit(2);
        }
    };

    match cli {
        cli::Cli::Statusline => {
            run_statusline();
            Ok(())
        }
        cli::Cli::Init { force } => commands::init::run(force),
        cli::Cli::Validate => commands::validate::run(),
        cli::Cli::Modules { name } => commands::modules_cmd::run(name),
        cli::Cli::Preview => commands::preview::run(),
        cli::Cli::Help => {
            cli::print_help();
            Ok(())
        }
        cli::Cli::Version => {
            cli::print_version();
            Ok(())
        }
    }
}

fn run_statusline() {
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
