pub mod term;

use crate::config::Config;
use crate::config::format::{Format, Node, contains_variable};
use crate::config::style::Style;
use crate::context::Context;
use crate::modules::Registry;
use std::collections::HashMap;
use std::fmt::Write as _;

use term::TermCaps;

/// Renders the top-level (global) format. Each `$ident` is resolved against
/// the module registry. A group containing variables is hidden if none of
/// its variables rendered anything; literal-only groups are always emitted.
pub fn render_global(format: &Format, ctx: &Context, cfg: &Config, registry: &Registry) -> String {
    let mut out = String::new();
    render_global_nodes(&format.0, ctx, cfg, registry, &mut out);
    out
}

fn render_global_nodes(
    nodes: &[Node],
    ctx: &Context,
    cfg: &Config,
    registry: &Registry,
    out: &mut String,
) -> bool {
    let mut any = false;
    for node in nodes {
        match node {
            Node::Literal(s) => out.push_str(s),
            Node::Variable(name) => {
                if let Some(module) = registry.get(name.as_str())
                    && let Some(s) = module.render(ctx, cfg)
                    && !s.is_empty()
                {
                    out.push_str(&s);
                    any = true;
                }
            }
            Node::Group { children, style } => {
                let mut buf = String::new();
                let any_child = render_global_nodes(children, ctx, cfg, registry, &mut buf);
                let has_var = contains_variable(children);
                if has_var && !any_child {
                    continue;
                }
                emit_group(&buf, style.as_deref(), &ctx.term, out);
                any = true;
            }
        }
    }
    any
}

/// Renders a per-module format string, substituting variables from a map.
/// The literal token `$style` inside a group's style position is substituted
/// with `default_style` (the module's configured style).
pub fn render_module(
    format_str: &str,
    default_style: &str,
    vars: &[(&str, String)],
    term: &TermCaps,
) -> String {
    let substituted = format_str.replace("$style", default_style);
    let format = match Format::parse(&substituted) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };
    let map: HashMap<&str, &str> = vars.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let mut out = String::new();
    render_module_nodes(&format.0, &map, term, &mut out);
    out
}

fn render_module_nodes(
    nodes: &[Node],
    vars: &HashMap<&str, &str>,
    term: &TermCaps,
    out: &mut String,
) -> bool {
    let mut any = false;
    for node in nodes {
        match node {
            Node::Literal(s) => out.push_str(s),
            Node::Variable(name) => {
                if let Some(v) = vars.get(name.as_str())
                    && !v.is_empty()
                {
                    out.push_str(v);
                    any = true;
                }
            }
            Node::Group { children, style } => {
                let mut buf = String::new();
                let any_child = render_module_nodes(children, vars, term, &mut buf);
                let has_var = contains_variable(children);
                if has_var && !any_child {
                    continue;
                }
                let resolved = style.as_deref().map(|s| substitute_style_vars(s, vars));
                emit_group(&buf, resolved.as_deref(), term, out);
                any = true;
            }
        }
    }
    any
}

/// Substitutes `$ident` references inside a style string with values from the
/// module's vars map. Unknown variables are replaced with the empty string —
/// so a missing `$h5_gradient_style` collapses cleanly instead of erroring.
fn substitute_style_vars(s: &str, vars: &HashMap<&str, &str>) -> String {
    if !s.contains('$') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '$' {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            if end > start {
                let name: String = chars[start..end].iter().collect();
                if let Some(v) = vars.get(name.as_str()) {
                    out.push_str(v);
                }
                i = end;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn emit_group(inner: &str, style_str: Option<&str>, term: &TermCaps, out: &mut String) {
    let style_str = style_str.map(|s| s.trim()).filter(|s| !s.is_empty());
    match style_str {
        Some(s) if term.enabled() => {
            let style = Style::parse(s).unwrap_or_default();
            let anstyle = style.to_anstyle(term);
            let _ = write!(
                out,
                "{}{}{}",
                anstyle.render(),
                inner,
                anstyle.render_reset()
            );
        }
        _ => out.push_str(inner),
    }
}
