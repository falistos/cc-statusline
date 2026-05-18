//! Terminal capabilities detection.
//!
//! Uses `anstyle-query` to detect truecolor support, the NO_COLOR convention,
//! and CLICOLOR_FORCE. We intentionally don't gate on isatty for stdout
//! because Claude Code captures stdout but the rendered ANSI escapes are
//! still re-emitted to the user's terminal — colors should be enabled
//! unless explicitly disabled.

#[derive(Debug, Clone, Copy)]
pub struct TermCaps {
    enabled: bool,
    truecolor: bool,
    ansi_256: bool,
}

impl TermCaps {
    pub fn detect() -> Self {
        if anstyle_query::no_color() {
            return Self::disabled();
        }
        let force = anstyle_query::clicolor_force();
        let term_color = anstyle_query::term_supports_color();
        let truecolor = anstyle_query::truecolor();
        let enabled = force || term_color || truecolor;
        Self {
            enabled,
            truecolor,
            ansi_256: enabled,
        }
    }

    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            truecolor: false,
            ansi_256: false,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn supports_truecolor(&self) -> bool {
        self.enabled && self.truecolor
    }
    pub fn supports_256(&self) -> bool {
        self.enabled && self.ansi_256
    }
}
