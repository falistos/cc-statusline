use super::Module;
use crate::config::Config;
use crate::config::schema::CacheHitScope;
use crate::context::Context;
use crate::render::render_module;

pub struct CacheHitModule;

impl Module for CacheHitModule {
    fn name(&self) -> &'static str {
        "cache_hit"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.cache_hit;
        if c.disabled {
            return None;
        }

        let (reads, creations, input) = match c.scope {
            CacheHitScope::Last => {
                let u = ctx.last_usage()?;
                (u.cache_read, u.cache_creation, u.input_tokens)
            }
            CacheHitScope::Session => {
                let s = ctx.full_stats()?;
                (
                    s.total_cache_read,
                    s.total_cache_creation,
                    s.total_input_tokens,
                )
            }
        };

        let denom = reads.saturating_add(creations).saturating_add(input);
        if denom == 0 {
            return None;
        }
        let pct = (reads as f64 / denom as f64) * 100.0;

        let vars = [
            ("pct", format!("{:.*}", c.precision, pct)),
            ("reads", reads.to_string()),
            ("creations", creations.to_string()),
            ("input", input.to_string()),
            ("total", denom.to_string()),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
