//! Module trait and static registry.
//!
//! Each module knows its name, reads its own config section, and produces
//! a fully-rendered string (with ANSI escapes embedded). A module returns
//! `None` to opt out of rendering — its slot in the global format is then
//! omitted entirely.

use crate::config::Config;
use crate::context::Context;
use std::collections::HashMap;

pub mod context;
pub mod cost;
pub mod model;
pub mod output_style;
pub mod rate_limits;
pub mod version;
pub mod workspace;

pub trait Module: Sync + Send {
    fn name(&self) -> &'static str;
    fn render(&self, ctx: &Context, cfg: &Config) -> Option<String>;
}

pub struct Registry {
    modules: HashMap<&'static str, Box<dyn Module>>,
}

impl Registry {
    pub fn new() -> Self {
        let mut modules: HashMap<&'static str, Box<dyn Module>> = HashMap::new();
        let all: Vec<Box<dyn Module>> = vec![
            Box::new(workspace::WorkspaceModule),
            Box::new(model::ModelModule),
            Box::new(context::ContextModule),
            Box::new(cost::CostModule),
            Box::new(rate_limits::RateLimitsModule),
            Box::new(output_style::OutputStyleModule),
            Box::new(version::VersionModule),
        ];
        for m in all {
            modules.insert(m.name(), m);
        }
        Self { modules }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Module> {
        self.modules.get(name).map(|b| b.as_ref())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
