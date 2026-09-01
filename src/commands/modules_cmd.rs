//! `modules` — list registered modules or describe one.

use crate::modules::Registry;
use anyhow::Result;

struct ModuleDoc {
    name: &'static str,
    summary: &'static str,
    variables: &'static [(&'static str, &'static str)],
}

const DOCS: &[ModuleDoc] = &[
    ModuleDoc {
        name: "workspace",
        summary: "Current working directory.",
        variables: &[
            ("path", "displayed path (home-relative + truncated)"),
            ("basename", "last segment"),
            ("full", "absolute path"),
        ],
    },
    ModuleDoc {
        name: "model",
        summary: "Active Claude model, reasoning effort and fast mode.",
        variables: &[
            ("name", "display name (with aliases applied)"),
            (
                "context_suffix",
                "\"1M\" when the 1M-context variant is active",
            ),
            ("id", "raw model id"),
            ("effort", "reasoning effort level"),
            ("fast", "\"fast\" when fast mode is on"),
        ],
    },
    ModuleDoc {
        name: "context",
        summary: "Context window usage. Source: stdin | transcript | auto.",
        variables: &[
            ("percent", "% used"),
            ("remaining", "% remaining"),
            ("bar", "filled bar (see [viz] for width/chars)"),
            ("spark", "single sparkline char ▁▂▃▄▅▆▇█"),
            ("circle", "single pie-meter char ○◔◑◕●"),
            (
                "gradient_style",
                "interpolated #rrggbb from [[context.gradient]]",
            ),
            ("tokens", "context tokens, short form (630k)"),
            ("window", "context window size, short form (1M)"),
            ("tokens_raw", "context tokens, exact"),
            ("window_raw", "context window size, exact"),
        ],
    },
    ModuleDoc {
        name: "cost",
        summary: "Session cost in USD.",
        variables: &[
            ("value", "USD with configured precision"),
            ("duration", "humanized total duration"),
            ("lines_added", "total lines added"),
            ("lines_removed", "total lines removed"),
            ("lines", "\"+added/-removed\", empty when nothing changed"),
        ],
    },
    ModuleDoc {
        name: "rate_limits",
        summary: "Usage windows merged from the payload, the last values seen and ~/.claude.json.",
        variables: &[
            ("h5 / d7", "5-hour and 7-day window %"),
            ("scoped", "model-scoped weekly window % (e.g. Fable)"),
            ("scoped_name", "model that scoped window belongs to"),
            ("spend", "gateway spend limit %"),
            ("credits", "extra-credit balance %"),
            ("<w>_bar", "filled bar for window <w>"),
            ("<w>_spark", "single sparkline char"),
            ("<w>_circle", "single pie-meter char"),
            ("<w>_style", "gradient or threshold style"),
            ("<w>_reset", "time until reset, when worth showing"),
            (
                "<w>_stale",
                "stale_symbol when the value is no longer fresh",
            ),
        ],
    },
    ModuleDoc {
        name: "prompt_cache",
        summary: "Prompt cache health. Needs Claude Code 2.1.251 or later.",
        variables: &[
            ("icon", "warm / expiring / cold symbol"),
            ("pct", "cache hit ratio %"),
            ("state", "\"warm\" or \"cold\""),
            ("warm", "\"warm\" when warm, else empty"),
            ("cold", "\"cold\" when cold, else empty"),
            ("expires_in", "time left before the prefix goes cold"),
            ("ttl", "cache lifetime of the current prefix (5m / 1h)"),
            ("requests", "API requests this session"),
            ("misses", "requests that re-processed cached content"),
            ("expiry_style", "style escalating as the TTL runs out"),
        ],
    },
    ModuleDoc {
        name: "session",
        summary: "Session name from /rename, --name or the generated title.",
        variables: &[("name", "session name (truncated)"), ("id", "session id")],
    },
    ModuleDoc {
        name: "output_style",
        summary: "Active output style. Hidden when default.",
        variables: &[("name", "style name")],
    },
    ModuleDoc {
        name: "version",
        summary: "Claude Code version (disabled by default).",
        variables: &[("version", "version string")],
    },
    ModuleDoc {
        name: "cache_hit",
        summary: "Cache read ratio. scope = last | session. Disabled by default.",
        variables: &[
            ("pct", "cache read percent"),
            ("reads", "cache_read tokens"),
            ("creations", "cache_creation tokens"),
            ("input", "fresh input tokens"),
            ("total", "denominator"),
        ],
    },
    ModuleDoc {
        name: "transcript_stats",
        summary: "Aggregate transcript counts. Disabled by default.",
        variables: &[
            ("messages", "user + assistant"),
            ("user", "user count"),
            ("assistant", "assistant count"),
            ("tools", "tool_use count"),
            ("input_tokens", "cumulative input"),
            ("output_tokens", "cumulative output"),
        ],
    },
    ModuleDoc {
        name: "tool_usage",
        summary: "Tool usage summary. Disabled by default.",
        variables: &[
            ("count", "total tool_use blocks"),
            ("last", "last tool name"),
        ],
    },
    ModuleDoc {
        name: "git_branch",
        summary: "Current git branch (raw .git/HEAD read).",
        variables: &[("branch", "branch name (or short SHA if detached)")],
    },
    ModuleDoc {
        name: "git_status",
        summary: "Git working tree status (shell-out + TTL cache).",
        variables: &[
            ("modified", "modified marker (with count if show_counts)"),
            ("untracked", "untracked marker"),
            ("added", "added marker"),
            ("deleted", "deleted marker"),
            ("renamed", "renamed marker"),
            ("conflicted", "conflicted marker"),
            ("ahead", "ahead marker"),
            ("behind", "behind marker"),
            ("clean", "clean marker (when fully clean)"),
        ],
    },
];

pub fn run(name: Option<String>) -> Result<()> {
    let registry = Registry::new();
    let known: std::collections::HashSet<&'static str> = registry.names().into_iter().collect();

    match name {
        Some(n) => {
            if !known.contains(n.as_str()) {
                anyhow::bail!("unknown module: {n}");
            }
            if let Some(doc) = DOCS.iter().find(|d| d.name == n) {
                println!("Module: {}\n  {}\n", doc.name, doc.summary);
                println!("  Variables:");
                for (v, d) in doc.variables {
                    println!("    ${v:<14} {d}");
                }
            } else {
                println!("Module: {n}  (no doc registered)");
            }
        }
        None => {
            println!("Available modules:\n");
            let mut docs: Vec<&ModuleDoc> =
                DOCS.iter().filter(|d| known.contains(d.name)).collect();
            docs.sort_by_key(|d| d.name);
            for d in docs {
                println!("  {:<18} {}", d.name, d.summary);
            }
            // Any modules registered but undocumented?
            for n in known {
                if !DOCS.iter().any(|d| d.name == n) {
                    println!("  {n:<18} (no doc)");
                }
            }
        }
    }
    Ok(())
}
