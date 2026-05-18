use super::Module;
use crate::config::Config;
use crate::context::Context;
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
            ("duration", humanize_duration_ms(duration_ms)),
            (
                "lines_added",
                info.total_lines_added.unwrap_or(0).to_string(),
            ),
            (
                "lines_removed",
                info.total_lines_removed.unwrap_or(0).to_string(),
            ),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn humanize_duration_ms(ms: u64) -> String {
    let total = ms / 1000;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}h{m:02}m")
    } else if m > 0 {
        format!("{m}m{s:02}s")
    } else {
        format!("{s}s")
    }
}
