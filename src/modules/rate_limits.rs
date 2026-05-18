use super::Module;
use super::context::{bar_opts, pick_gradient_or_threshold};
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;
use crate::viz;

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

        let opts = bar_opts(&cfg.viz);
        let (h5_str, h5_bar, h5_spark, h5_circle, h5_style) = if h5_show {
            let p = h5.unwrap();
            (
                format!("{p:.0}"),
                viz::bar(p, &opts),
                viz::spark(p).to_string(),
                viz::circle(p).to_string(),
                pick_gradient_or_threshold(&c.gradient, &c.thresholds, &c.style, p),
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                c.style.clone(),
            )
        };
        let (d7_str, d7_bar, d7_spark, d7_circle, d7_style) = if d7_show {
            let p = d7.unwrap();
            (
                format!("{p:.0}"),
                viz::bar(p, &opts),
                viz::spark(p).to_string(),
                viz::circle(p).to_string(),
                pick_gradient_or_threshold(&c.gradient, &c.thresholds, &c.style, p),
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                c.style.clone(),
            )
        };

        let vars = [
            ("h5", h5_str),
            ("h5_bar", h5_bar),
            ("h5_spark", h5_spark),
            ("h5_circle", h5_circle),
            ("h5_style", h5_style),
            ("d7", d7_str),
            ("d7_bar", d7_bar),
            ("d7_spark", d7_spark),
            ("d7_circle", d7_circle),
            ("d7_style", d7_style),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
