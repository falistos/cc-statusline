use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct TranscriptStatsModule;

impl Module for TranscriptStatsModule {
    fn name(&self) -> &'static str {
        "transcript_stats"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.transcript_stats;
        if c.disabled {
            return None;
        }
        let s = ctx.full_stats()?;
        let total = s.user_messages + s.assistant_messages;
        let vars = [
            ("messages", total.to_string()),
            ("user", s.user_messages.to_string()),
            ("assistant", s.assistant_messages.to_string()),
            ("tools", s.tool_uses.to_string()),
            ("input_tokens", s.total_input_tokens.to_string()),
            ("output_tokens", s.total_output_tokens.to_string()),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
