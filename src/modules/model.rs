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

        let (name, context_suffix) = match resolved.strip_suffix(" (1M context)") {
            Some(base) if m.strip_context_suffix => (base.to_string(), "1M".to_string()),
            _ => (resolved.to_string(), String::new()),
        };
        let vars = [
            ("name", name),
            ("context_suffix", context_suffix),
            ("id", input.id.clone().unwrap_or_default()),
            (
                "effort",
                ctx.input
                    .effort
                    .as_ref()
                    .and_then(|e| e.level.clone())
                    .unwrap_or_default(),
            ),
            (
                "fast",
                if ctx.input.fast_mode.unwrap_or(false) {
                    "fast".to_string()
                } else {
                    String::new()
                },
            ),
        ];
        let out = render_module(&m.format, &m.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
