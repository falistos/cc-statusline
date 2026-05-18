use super::Module;
use crate::config::Config;
use crate::config::schema::Threshold;
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
        let pct = ctx.input.context_window.as_ref()?.used_percentage?;
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

/// Pick the threshold with the smallest `max` such that `value <= max`.
/// Falls back to `default` if no threshold matches.
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
