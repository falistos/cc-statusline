use super::Module;
use super::context::{bar_opts, pick_gradient_or_threshold};
use crate::config::Config;
use crate::config::schema::RateLimitsConfig;
use crate::context::Context;
use crate::render::render_module;
use crate::usage::{self, Window, WindowKind};
use crate::viz;

pub struct RateLimitsModule;

impl Module for RateLimitsModule {
    fn name(&self) -> &'static str {
        "rate_limits"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.rate_limits;
        if c.disabled {
            return None;
        }

        let windows = usage::windows(
            &ctx.input,
            c.use_cached_snapshot,
            c.snapshot_ttl_seconds,
            ctx.persist_usage,
        );
        let now = usage::now();
        let mut named: Vec<(String, String)> = Vec::with_capacity(windows.len() * 8);

        for window in &windows {
            if window.percent < c.hide_below_percent {
                continue;
            }
            let prefix = match window.kind {
                WindowKind::FiveHour => "h5",
                WindowKind::SevenDay => "d7",
                WindowKind::Scoped => "scoped",
                WindowKind::Spend => "spend",
                WindowKind::Credits => "credits",
            };
            push_window_vars(&mut named, prefix, window, c, cfg, now);
        }
        if named.is_empty() {
            return None;
        }

        let vars: Vec<(&str, String)> = named
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
            .collect();
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn push_window_vars(
    vars: &mut Vec<(String, String)>,
    prefix: &str,
    window: &Window,
    c: &RateLimitsConfig,
    cfg: &Config,
    now: u64,
) {
    let p = window.percent;
    let style = pick_gradient_or_threshold(&c.gradient, &c.thresholds, &c.style, p);
    let stale = if window.age > c.stale_after_seconds {
        c.stale_symbol.clone()
    } else {
        String::new()
    };

    // A countdown only earns its place once the window fills up or is about
    // to roll over; the rest of the time it is noise.
    let reset = window
        .resets_at
        .map(|at| at.saturating_sub(now))
        .filter(|&left| p >= c.reset_above_percent || left < c.reset_within_seconds)
        .map(humanize_duration)
        .unwrap_or_default();

    for (suffix, value) in [
        ("", format!("{p:.0}")),
        ("_bar", viz::bar(p, &bar_opts(&cfg.viz))),
        ("_spark", viz::spark(p).to_string()),
        ("_circle", viz::circle(p).to_string()),
        ("_style", style),
        ("_reset", reset),
        ("_stale", stale),
        ("_name", window.label.clone()),
    ] {
        vars.push((format!("{prefix}{suffix}"), value));
    }
}

fn humanize_duration(secs: u64) -> String {
    match secs {
        s if s >= 86_400 => format!("{}d{}h", s / 86_400, (s % 86_400) / 3_600),
        s if s >= 3_600 => format!("{}h{:02}", s / 3_600, (s % 3_600) / 60),
        s if s >= 60 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}
