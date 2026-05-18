use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct OutputStyleModule;

impl Module for OutputStyleModule {
    fn name(&self) -> &'static str {
        "output_style"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.output_style;
        if c.disabled {
            return None;
        }
        let name = ctx.input.output_style.as_ref()?.name.as_deref()?;
        if name.is_empty() {
            return None;
        }
        if c.hide_if_default && name.eq_ignore_ascii_case("default") {
            return None;
        }
        let vars = [("name", name.to_string())];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
