use super::Module;
use crate::config::Config;
use crate::context::Context;
use crate::render::render_module;
use std::path::Path;

pub struct WorkspaceModule;

impl Module for WorkspaceModule {
    fn name(&self) -> &'static str {
        "workspace"
    }

    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String> {
        let w = &cfg.workspace;
        if w.disabled {
            return None;
        }

        let (current, project) = current_and_project(ctx);
        let current = current?;
        let path = format_path(current, w.truncate, &w.truncate_symbol);
        let basename = Path::new(current)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(current)
            .to_string();

        let is_subdir = match project {
            Some(p) => current != p,
            None => false,
        };
        let effective_style = if is_subdir && !w.style_subdir.is_empty() {
            w.style_subdir.as_str()
        } else {
            w.style.as_str()
        };

        let vars = [
            ("path", path),
            ("basename", basename),
            ("full", current.to_string()),
        ];
        let out = render_module(&w.format, effective_style, &vars, &ctx.term);
        if out.is_empty() { None } else { Some(out) }
    }
}

fn current_and_project(ctx: &Context) -> (Option<&str>, Option<&str>) {
    let current = ctx
        .input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref())
        .or(ctx.input.cwd.as_deref());
    let project = ctx
        .input
        .workspace
        .as_ref()
        .and_then(|w| w.project_dir.as_deref());
    (current, project)
}

/// Keep the trailing `n` segments of a path. `n == 0` disables truncation.
fn format_path(path: &str, truncate: usize, symbol: &str) -> String {
    if truncate == 0 {
        return path.to_string();
    }
    // Special-case "~" prefix for home: show "~/..." rather than "/Users/...".
    let displayed = home_relative(path);

    let segments: Vec<&str> = displayed.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() <= truncate {
        return displayed;
    }
    let kept = &segments[segments.len() - truncate..];
    format!("{symbol}{}", kept.join("/"))
}

fn home_relative(path: &str) -> String {
    if let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())
        && let Some(home_str) = home.to_str()
        && let Some(rest) = path.strip_prefix(home_str)
    {
        let rest = rest.trim_start_matches('/');
        return if rest.is_empty() {
            "~".to_string()
        } else {
            format!("~/{rest}")
        };
    }
    path.to_string()
}
