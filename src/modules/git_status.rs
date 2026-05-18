use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::git;
use crate::render::render_module;
use std::path::Path;

pub struct GitStatusModule;

impl Module for GitStatusModule {
    fn name(&self) -> &'static str {
        "git_status"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let c = &cfg.git_status;
        if c.disabled {
            return None;
        }
        let cwd = ctx
            .input
            .workspace
            .as_ref()
            .and_then(|w| w.current_dir.as_deref())
            .or(ctx.input.cwd.as_deref())?;
        let status = git::read_status(Path::new(cwd), c.cache_ttl_seconds)?;

        let counted = |sym: &str, n: u32| -> String {
            if n == 0 {
                String::new()
            } else if c.show_counts {
                format!("{sym}{n}")
            } else {
                sym.to_string()
            }
        };

        let vars = [
            ("modified", counted(&c.modified_symbol, status.modified)),
            ("untracked", counted(&c.untracked_symbol, status.untracked)),
            ("added", counted(&c.added_symbol, status.added)),
            ("deleted", counted(&c.deleted_symbol, status.deleted)),
            ("renamed", counted(&c.renamed_symbol, status.renamed)),
            (
                "conflicted",
                counted(&c.conflicted_symbol, status.conflicted),
            ),
            ("ahead", counted(&c.ahead_symbol, status.ahead)),
            ("behind", counted(&c.behind_symbol, status.behind)),
            (
                "clean",
                if status.is_clean() {
                    c.clean_symbol.clone()
                } else {
                    String::new()
                },
            ),
        ];
        let out = render_module(&c.format, &c.style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}
