use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct ToolUsageModule;

impl Module for ToolUsageModule {
    fn name(&self) -> &'static str {
        "tool_usage"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.tool_usage;
        if c.disabled {
            return None;
        }
        let s = ctx.full_stats()?;
        if s.tool_uses == 0 {
            return None;
        }
        let last = s.last_tool_name.clone().unwrap_or_default();
        let vars = [("count", s.tool_uses.to_string()), ("last", last)];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
