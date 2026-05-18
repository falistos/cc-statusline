//! Command-line argument parsing with lexopt.

use anyhow::Result;
use lexopt::Arg;

#[derive(Debug)]
pub enum Cli {
    /// Default mode: read JSON from stdin and emit the statusline.
    Statusline,
    Init {
        force: bool,
    },
    Validate,
    Modules {
        name: Option<String>,
    },
    Preview,
    Help,
    Version,
}

const HELP: &str = "\
cc-statusline — fast, configurable statusline for Claude Code

USAGE
  cc-statusline                        Run as statusline (reads stdin JSON, writes to stdout).
  cc-statusline init [--force]         Create a config file and patch ~/.claude/settings.json.
  cc-statusline validate               Validate the current config.
  cc-statusline modules                List available modules.
  cc-statusline modules <NAME>         Show variables exposed by a module.
  cc-statusline preview                Render using a mock payload.
  cc-statusline -h | --help            Show this help.
  cc-statusline -V | --version         Show version.
";

pub fn parse() -> Result<Cli> {
    let mut parser = lexopt::Parser::from_env();
    let first = parser.next().map_err(|e| anyhow::anyhow!(e))?;
    let Some(first) = first else {
        return Ok(Cli::Statusline);
    };

    match first {
        Arg::Short('h') | Arg::Long("help") => Ok(Cli::Help),
        Arg::Short('V') | Arg::Long("version") => Ok(Cli::Version),
        Arg::Value(s) => {
            let sub = s.to_string_lossy().to_string();
            match sub.as_str() {
                "init" => {
                    let mut force = false;
                    while let Some(arg) = parser.next().map_err(|e| anyhow::anyhow!(e))? {
                        match arg {
                            Arg::Long("force") | Arg::Short('f') => force = true,
                            _ => anyhow::bail!("unexpected argument for `init`"),
                        }
                    }
                    Ok(Cli::Init { force })
                }
                "validate" => Ok(Cli::Validate),
                "modules" => {
                    let next = parser.next().map_err(|e| anyhow::anyhow!(e))?;
                    let name = match next {
                        Some(Arg::Value(v)) => Some(v.to_string_lossy().to_string()),
                        Some(_) => anyhow::bail!("unexpected option after `modules`"),
                        None => None,
                    };
                    Ok(Cli::Modules { name })
                }
                "preview" => Ok(Cli::Preview),
                other => anyhow::bail!("unknown subcommand: {other}"),
            }
        }
        _ => anyhow::bail!("unexpected argument"),
    }
}

pub fn print_help() {
    println!("{HELP}");
}

pub fn print_version() {
    println!("cc-statusline {}", env!("CARGO_PKG_VERSION"));
}
