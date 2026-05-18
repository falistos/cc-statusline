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
        summary: "Active Claude model.",
        variables: &[
            ("name", "display name (with aliases applied)"),
            ("id", "raw model id"),
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
            ("gradient_style", "interpolated #rrggbb from [[context.gradient]]"),
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
        ],
    },
    ModuleDoc {
        name: "rate_limits",
        summary: "5h and 7d usage percentages (independently collapsible).",
        variables: &[
            ("h5", "5-hour window %"),
            ("h5_bar", "5h filled bar"),
            ("h5_spark", "5h single sparkline char"),
            ("h5_circle", "5h pie-meter char"),
            ("h5_style", "5h gradient or threshold style"),
            ("d7", "7-day window %"),
            ("d7_bar", "7d filled bar"),
            ("d7_spark", "7d single sparkline char"),
            ("d7_circle", "7d pie-meter char"),
            ("d7_style", "7d gradient or threshold style"),
        ],
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
