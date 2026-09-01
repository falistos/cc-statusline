use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;
use crate::usage;

pub struct PromptCacheModule;

impl Module for PromptCacheModule {
    fn name(&self) -> &'static str {
        "prompt_cache"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.prompt_cache;
        if c.disabled {
            return None;
        }
        // Absent before Claude Code 2.1.251, and when the gateway strips
        // cache token counts: nothing to say in either case.
        let pc = ctx.input.prompt_cache.as_ref()?;
        if !pc.caching_observed.unwrap_or(false) {
            return None;
        }

        let ratio = pc
            .hit_ratio
            .map(|r| format!("{:.0}", r * 100.0))
            .unwrap_or_default();
        let warm = pc.warm.unwrap_or(false);
        let left = pc
            .expires_at
            .filter(|_| warm)
            .map(|at| at.saturating_sub(usage::now()));

        // Losing a warm prefix means the next request pays full price, so both
        // the icon and the countdown escalate as the TTL runs out.
        let (expiry_style, icon) = match (warm, left) {
            (false, _) => (&c.cold_style, &c.cold_symbol),
            (true, Some(s)) if s <= c.alert_seconds => (&c.alert_style, &c.alert_symbol),
            (true, Some(s)) if s <= c.warn_seconds => (&c.warn_style, &c.warm_symbol),
            (true, _) => (&c.warm_style, &c.warm_symbol),
        };

        let vars = [
            ("icon", icon.clone()),
            ("pct", ratio),
            ("state", if warm { "warm" } else { "cold" }.to_string()),
            (
                "warm",
                if warm {
                    "warm".to_string()
                } else {
                    String::new()
                },
            ),
            (
                "cold",
                if warm {
                    String::new()
                } else {
                    "cold".to_string()
                },
            ),
            ("expires_in", left.map(humanize).unwrap_or_default()),
            ("ttl", pc.ttl.clone().unwrap_or_default()),
            (
                "requests",
                pc.requests.map(|v| v.to_string()).unwrap_or_default(),
            ),
            (
                "misses",
                pc.misses.map(|v| v.to_string()).unwrap_or_default(),
            ),
            ("expiry_style", expiry_style.clone()),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn humanize(secs: u64) -> String {
    match secs {
        s if s >= 3_600 => format!("{}h{:02}", s / 3_600, (s % 3_600) / 60),
        s if s >= 60 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}
