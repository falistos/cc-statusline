use super::Module;
use super::context::pick_threshold_style;
use crate::config::Config;
use crate::context::Context;
use crate::fmt;
use crate::render::render_module;

pub struct CostModule;

impl Module for CostModule {
    fn name(&self) -> &'static str {
        "cost"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.cost;
        if c.disabled {
            return None;
        }
        let info = ctx.input.cost.as_ref()?;
        let usd = info.total_cost_usd?;
        if usd < c.hide_below {
            return None;
        }
        let duration_ms = info.total_duration_ms.unwrap_or(0);
        let vars = [
            ("value", format!("{:.*}", c.precision, usd)),
            ("duration", fmt::duration(duration_ms / 1000)),
            (
                "lines_added",
                info.total_lines_added.unwrap_or(0).to_string(),
            ),
            (
                "lines_removed",
                info.total_lines_removed.unwrap_or(0).to_string(),
            ),
            ("lines", lines_summary(info)),
        ];
        let style = pick_threshold_style(&c.thresholds, &c.style, usd);
        let out = render_module(&c.format, &style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn lines_summary(info: &crate::input::Cost) -> String {
    let added = info.total_lines_added.unwrap_or(0);
    let removed = info.total_lines_removed.unwrap_or(0);
    if added + removed == 0 {
        return String::new();
    }
    format!("+{added}/-{removed}")
}
