use crate::input::ClaudeInput;
use crate::render::term::TermCaps;

pub struct Context {
    pub input: ClaudeInput,
    pub term: TermCaps,
}

impl Context {
    pub fn new(input: ClaudeInput) -> Self {
        Self {
            input,
            term: TermCaps::detect(),
        }
    }
}
