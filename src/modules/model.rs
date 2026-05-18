use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct ModelModule;

impl Module for ModelModule {
    fn name(&self) -> &'static str {
        "model"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let m = &cfg.model;
        if m.disabled {
            return None;
        }
        let input = ctx.input.model.as_ref()?;
        let name = input.display_name.as_deref().unwrap_or_default();
        if name.is_empty() {
            return None;
        }
        let resolved = m
            .aliases
            .get(name)
            .map(String::as_str)
            .or_else(|| {
                input
                    .id
                    .as_deref()
                    .and_then(|id| m.aliases.get(id).map(String::as_str))
            })
            .unwrap_or(name);

        let vars = [
            ("name", resolved.to_string()),
            ("id", input.id.clone().unwrap_or_default()),
        ];
        let out = render_module(&m.format, &m.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
