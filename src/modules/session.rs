use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct SessionModule;

impl Module for SessionModule {
    fn name(&self) -> &'static str {
        "session"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.session;
        if c.disabled {
            return None;
        }
        let name = ctx.input.session_name.as_deref()?.trim();
        if name.is_empty() {
            return None;
        }
        let shown = truncate(name, c.truncate, &c.truncate_symbol);
        let vars = [
            ("name", shown),
            ("id", ctx.input.session_id.clone().unwrap_or_default()),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn truncate(s: &str, max: usize, symbol: &str) -> String {
    if max == 0 || s.chars().count() <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(symbol.chars().count());
    s.chars().take(keep).collect::<String>() + symbol
}
