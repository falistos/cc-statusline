//! `validate` — parse the user's config and lint references in `format`.

use crate::config::Format;
use crate::config::format::Node;
use crate::modules::Registry;
use anyhow::Result;
use std::collections::BTreeSet;

pub fn run() -> Result<()> {
    let cfg = match crate::config::load() {
        Ok(c) => {
            println!("✓ Config parsed.");
            c
        }
        Err(e) => {
            eprintln!("✗ Failed to load config: {e:#}");
            return Err(e);
        }
    };

    let format = Format::parse(&cfg.format)?;
    println!("✓ Global format syntax OK.");

    let mut referenced = BTreeSet::new();
    collect_variables(&format.0, &mut referenced);

    let registry = Registry::new();
    let mut unknown = Vec::new();
    for name in &referenced {
        if registry.get(name).is_none() {
            unknown.push(name.clone());
        }
    }

    if unknown.is_empty() {
        println!(
            "✓ All referenced modules exist ({} used).",
            referenced.len()
        );
    } else {
        eprintln!(
            "⚠ Unknown modules referenced in format: {}",
            unknown.join(", ")
        );
    }

    let all_modules: BTreeSet<&'static str> = registry.names().into_iter().collect();
    let unused: Vec<&&'static str> = all_modules
        .iter()
        .filter(|n| !referenced.contains(**n))
        .collect();
    if !unused.is_empty() {
        println!(
            "ℹ Modules not referenced in format (will never render): {}",
            unused.iter().map(|s| **s).collect::<Vec<_>>().join(", ")
        );
    }

    Ok(())
}

fn collect_variables(nodes: &[Node], out: &mut BTreeSet<String>) {
    for n in nodes {
        match n {
            Node::Variable(name) => {
                out.insert(name.clone());
            }
            Node::Group { children, .. } => collect_variables(children, out),
            Node::Literal(_) => {}
        }
    }
}
