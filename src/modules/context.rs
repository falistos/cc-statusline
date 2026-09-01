use super::Module;
use crate::config::Config;
use crate::config::schema::{ContextSource, GradientStop as CfgStop, Threshold, VizConfig};
use crate::context::Context;
use crate::render::render_module;
use crate::viz;

pub struct ContextModule;

impl Module for ContextModule {
    fn name(&self) -> &'static str {
        "context"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.context;
        if c.disabled {
            return None;
        }

        let pct = match c.source {
            ContextSource::Stdin => stdin_percent(ctx)?,
            ContextSource::Transcript => transcript_percent(ctx, c.default_window_size)?,
            ContextSource::Auto => stdin_percent(ctx)
                .filter(|p| (0.01..=100.0).contains(p))
                .or_else(|| transcript_percent(ctx, c.default_window_size))?,
        };

        let cw = ctx.input.context_window.as_ref();
        let tokens = cw.and_then(|c| c.total_input_tokens);
        let window = cw.and_then(|c| c.context_window_size);
        let style = pick_threshold_style(&c.thresholds, &c.style, pct);
        let gradient_style = pick_gradient_or_threshold(&c.gradient, &c.thresholds, &c.style, pct);
        let vars = [
            ("percent", format!("{:.*}", c.precision, pct)),
            (
                "remaining",
                format!("{:.*}", c.precision, (100.0 - pct).max(0.0)),
            ),
            ("bar", viz::bar(pct, &bar_opts(&cfg.viz))),
            ("spark", viz::spark(pct).to_string()),
            ("circle", viz::circle(pct).to_string()),
            ("gradient_style", gradient_style),
            ("tokens", short_tokens(tokens)),
            ("window", short_tokens(window)),
            (
                "tokens_raw",
                tokens.map(|t| t.to_string()).unwrap_or_default(),
            ),
            (
                "window_raw",
                window.map(|t| t.to_string()).unwrap_or_default(),
            ),
        ];
        let out = render_module(&c.format, &style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

pub(crate) fn bar_opts(v: &VizConfig) -> viz::BarOpts<'_> {
    viz::BarOpts {
        width: v.bar_width,
        filled: &v.bar_filled,
        empty: &v.bar_empty,
        partial: v.bar_partial,
    }
}

pub(crate) fn pick_gradient_or_threshold(
    gradient: &[CfgStop],
    thresholds: &[Threshold],
    default: &str,
    value: f64,
) -> String {
    if !gradient.is_empty() {
        let stops: Vec<viz::GradientStop> = gradient
            .iter()
            .filter_map(|s| {
                viz::parse_hex(&s.color).map(|(r, g, b)| viz::GradientStop { at: s.at, r, g, b })
            })
            .collect();
        if !stops.is_empty() {
            return viz::gradient_hex(value, &stops);
        }
    }
    pick_threshold_style(thresholds, default, value)
}

/// 630123 -> "630k", 1000000 -> "1M".
fn short_tokens(t: Option<u64>) -> String {
    match t {
        Some(t) if t >= 1_000_000 && t % 1_000_000 == 0 => format!("{}M", t / 1_000_000),
        Some(t) if t >= 1_000_000 => format!("{}.{}M", t / 1_000_000, (t % 1_000_000) / 100_000),
        Some(t) if t >= 1_000 => format!("{}k", t / 1_000),
        Some(t) => t.to_string(),
        None => String::new(),
    }
}

fn stdin_percent(ctx: &Context) -> Option<f64> {
    ctx.input.context_window.as_ref()?.used_percentage
}

fn transcript_percent(ctx: &Context, default_size: u64) -> Option<f64> {
    let usage = ctx.last_usage()?;
    let window = ctx
        .input
        .context_window
        .as_ref()
        .and_then(|c| c.context_window_size)
        .unwrap_or(default_size);
    if window == 0 {
        return None;
    }
    Some((usage.context_tokens() as f64 / window as f64) * 100.0)
}

pub(crate) fn pick_threshold_style(thresholds: &[Threshold], default: &str, value: f64) -> String {
    thresholds
        .iter()
        .filter(|t| value <= t.max)
        .min_by(|a, b| {
            a.max
                .partial_cmp(&b.max)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|t| t.style.clone())
        .unwrap_or_else(|| default.to_string())
}
