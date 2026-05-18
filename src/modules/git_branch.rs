use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::git;
use crate::render::render_module;
use std::path::Path;

pub struct GitBranchModule;

impl Module for GitBranchModule {
    fn name(&self) -> &'static str {
        "git_branch"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.git_branch;
        if c.disabled {
            return None;
        }
        let cwd = ctx
            .input
            .workspace
            .as_ref()
            .and_then(|w| w.current_dir.as_deref())
            .or(ctx.input.cwd.as_deref())?;
        let branch = git::read_branch(Path::new(cwd))?;
        let branch = truncate(&branch, c.truncate, &c.truncate_symbol);
        let vars = [("branch", branch)];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn truncate(s: &str, max: usize, symbol: &str) -> String {
    if max == 0 || s.chars().count() <= max {
        return s.to_string();
    }
    let kept: String = s.chars().take(max).collect();
    format!("{kept}{symbol}")
}
