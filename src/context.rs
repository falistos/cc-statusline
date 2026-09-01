use crate::input::ClaudeInput;
use crate::render::term::TermCaps;
use crate::transcript::{TranscriptStats, TranscriptUsage, read_full_stats, read_last_usage};
use std::path::Path;
use std::sync::OnceLock;

pub struct Context {
    pub input: ClaudeInput,
    pub term: TermCaps,
    /// False in `preview`, so a mock payload never overwrites the usage
    /// windows persisted from real sessions.
    pub persist_usage: bool,
    last_usage: OnceLock<Option<TranscriptUsage>>,
    full_stats: OnceLock<Option<TranscriptStats>>,
}

impl Context {
    pub fn new(input: ClaudeInput) -> Self {
        Self {
            input,
            term: TermCaps::detect(),
            persist_usage: true,
            last_usage: OnceLock::new(),
            full_stats: OnceLock::new(),
        }
    }

    pub fn preview(input: ClaudeInput) -> Self {
        Self {
            persist_usage: false,
            ..Self::new(input)
        }
    }

    /// Returns the usage of the most recent assistant message in the
    /// transcript, if any. Read lazily and cached.
    pub fn last_usage(&self) -> Option<&TranscriptUsage> {
        self.last_usage
            .get_or_init(|| {
                let path = self.input.transcript_path.as_deref()?;
                read_last_usage(Path::new(path))
            })
            .as_ref()
    }

    /// Returns aggregated stats over the full transcript. Triggers a
    /// full file read on first call.
    pub fn full_stats(&self) -> Option<&TranscriptStats> {
        self.full_stats
            .get_or_init(|| {
                let path = self.input.transcript_path.as_deref()?;
                read_full_stats(Path::new(path))
            })
            .as_ref()
    }
}
