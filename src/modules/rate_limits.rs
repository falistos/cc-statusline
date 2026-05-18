use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;

pub struct RateLimitsModule;

impl Module for RateLimitsModule {
    fn name(&self) -> &'static str {
        "rate_limits"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.rate_limits;
        if c.disabled {
            return None;
        }
        let rl = ctx.input.rate_limits.as_ref()?;
        let h5 = rl.five_hour.as_ref().and_then(|w| w.used_percentage);
        let d7 = rl.seven_day.as_ref().and_then(|w| w.used_percentage);

        let h5_show = h5.is_some_and(|p| p >= c.hide_below_percent);
        let d7_show = d7.is_some_and(|p| p >= c.hide_below_percent);
        if !h5_show && !d7_show {
            return None;
        }

        let h5_str = if h5_show {
            format!("{:.0}", h5.unwrap())
        } else {
            String::new()
        };
        let d7_str = if d7_show {
            format!("{:.0}", d7.unwrap())
        } else {
            String::new()
        };

        let vars = [("h5", h5_str), ("d7", d7_str)];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
