use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct VersionModule;

impl Module for VersionModule {
    fn name(&self) -> &'static str {
        "version"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.version;
        if c.disabled {
            return None;
        }
        let v = ctx.input.version.as_deref()?;
        if v.is_empty() {
            return None;
        }
        let vars = [("version", v.to_string())];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
