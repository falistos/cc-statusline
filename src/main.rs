use std::io::{self, Read};
use std::panic;
use std::process::ExitCode;

mod cache;
mod cli;
mod commands;
mod config;
mod context;
mod errors;
mod git;
mod input;
mod modules;
mod paths;
mod render;
mod transcript;

use config::Format;
use context::Context;
use input::ClaudeInput;
use modules::Registry;

fn main() -> ExitCode {
    panic::set_hook(Box::new(|info| {
        // Statusline must never crash visibly: swallow panics so Claude Code
        // keeps the previous statusline rather than rendering an error.
        // We persist the panic to the error log so it's diagnosable later.
        errors::log("panic", &info.to_string());
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
    if let Err(e) = io::stdin().read_to_string(&mut buf) {
        errors::log("stdin read", &e.to_string());
        return;
    }
    let input: ClaudeInput = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(e) => {
            let sample: String = buf.chars().take(500).collect();
            errors::log("stdin parse", &format!("{e} | stdin[0..500]: {sample}"));
            return;
        }
    };
    let cfg = match config::load() {
        Ok(c) => c,
        Err(e) => {
            errors::log("config load", &e.to_string());
            config::Config::default()
        }
    };
    let format = match Format::parse(&cfg.format) {
        Ok(f) => f,
        Err(e) => {
            errors::log("format parse", &format!("{e} | format: {}", cfg.format));
            Format(vec![])
        }
    };
    let ctx = Context::new(input);
    let registry = Registry::new();
    let rendered = render::render_global(&format, &ctx, &cfg, &registry);
    print!("{rendered}");
}
