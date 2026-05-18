use super::Module;
use crate::config::Config;
use crate::config::schema::{ContextSource, Threshold};
use crate::context::Context;
use crate::render::render_module;

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

        let style = pick_threshold_style(&c.thresholds, &c.style, pct);
        let vars = [
            ("percent", format!("{:.*}", c.precision, pct)),
            (
                "remaining",
                format!("{:.*}", c.precision, (100.0 - pct).max(0.0)),
            ),
        ];
        let out = render_module(&c.format, &style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
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

fn pick_threshold_style(thresholds: &[Threshold], default: &str, value: f64) -> String {
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
