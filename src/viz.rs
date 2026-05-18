//! Visual renderings for percentage values: bars, sparklines, circles, and
//! gradient color interpolation.
//!
//! All inputs are clamped to `0.0..=100.0`. Outputs are plain strings — no
//! ANSI escapes; coloring is applied by the format engine via the module's
//! resolved style.

/// Sub-cell eighths used for smooth bar fills: `▏▎▍▌▋▊▉█`.
const EIGHTHS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

/// Sparkline glyphs from 0 to 100% in eight steps.
const SPARK: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Pie/circle meter from empty to full in five steps.
const CIRCLE: [char; 5] = ['○', '◔', '◑', '◕', '●'];

#[derive(Debug, Clone)]
pub struct BarOpts<'a> {
    pub width: usize,
    pub filled: &'a str,
    pub empty: &'a str,
    /// When true, use sub-cell eighths for the partially-filled cell so the
    /// bar fills smoothly. When false, each cell is either fully filled or
    /// fully empty.
    pub partial: bool,
}

pub fn bar(pct: f64, opts: &BarOpts) -> String {
    if opts.width == 0 {
        return String::new();
    }
    let pct = pct.clamp(0.0, 100.0) / 100.0;

    if opts.partial && opts.filled == "█" && opts.empty == "░" {
        return bar_eighths(pct, opts.width);
    }

    let total_cells = opts.width;
    let fill_cells = (pct * total_cells as f64).round() as usize;
    let fill_cells = fill_cells.min(total_cells);
    let mut s = String::with_capacity(total_cells * 3);
    for _ in 0..fill_cells {
        s.push_str(opts.filled);
    }
    for _ in fill_cells..total_cells {
        s.push_str(opts.empty);
    }
    s
}

fn bar_eighths(pct: f64, width: usize) -> String {
    // 8 sub-steps per cell; total steps = width * 8.
    let total_steps = width * 8;
    let steps = (pct * total_steps as f64).round() as usize;
    let full_cells = steps / 8;
    let remainder = steps % 8;
    let mut s = String::with_capacity(width * 3);
    for _ in 0..full_cells {
        s.push('█');
    }
    if full_cells < width {
        if remainder > 0 {
            s.push(EIGHTHS[remainder - 1]);
            for _ in (full_cells + 1)..width {
                s.push('░');
            }
        } else {
            for _ in full_cells..width {
                s.push('░');
            }
        }
    }
    s
}

pub fn spark(pct: f64) -> char {
    let pct = pct.clamp(0.0, 100.0);
    // Map 0..=100 to 0..=7. Anything > 0 gets at least ▁ so the user sees
    // *something* rather than nothing.
    if pct <= 0.0 {
        return SPARK[0];
    }
    let idx = ((pct / 100.0) * (SPARK.len() as f64 - 1.0)).round() as usize;
    SPARK[idx.min(SPARK.len() - 1)]
}

pub fn circle(pct: f64) -> char {
    let pct = pct.clamp(0.0, 100.0);
    let idx = ((pct / 100.0) * (CIRCLE.len() as f64 - 1.0)).round() as usize;
    CIRCLE[idx.min(CIRCLE.len() - 1)]
}

#[derive(Debug, Clone)]
pub struct GradientStop {
    pub at: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Interpolates between stops and returns a `#rrggbb` string suitable for
/// embedding in a style string. Returns an empty string if no stops are given.
/// Stops must be sorted by `at` (caller's responsibility).
pub fn gradient_hex(pct: f64, stops: &[GradientStop]) -> String {
    if stops.is_empty() {
        return String::new();
    }
    let pct = pct.clamp(0.0, 100.0);

    if pct <= stops[0].at {
        return format_hex(stops[0].r, stops[0].g, stops[0].b);
    }
    if let Some(last) = stops.last()
        && pct >= last.at
    {
        return format_hex(last.r, last.g, last.b);
    }

    for pair in stops.windows(2) {
        let lo = &pair[0];
        let hi = &pair[1];
        if pct >= lo.at && pct <= hi.at {
            let span = (hi.at - lo.at).max(f64::EPSILON);
            let t = (pct - lo.at) / span;
            let r = lerp(lo.r, hi.r, t);
            let g = lerp(lo.g, hi.g, t);
            let b = lerp(lo.b, hi.b, t);
            return format_hex(r, g, b);
        }
    }
    format_hex(stops[0].r, stops[0].g, stops[0].b)
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    let v = a as f64 + (b as f64 - a as f64) * t;
    v.round().clamp(0.0, 255.0) as u8
}

fn format_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Parses `"#rrggbb"` or `"#rgb"` into an `(r, g, b)` triple. Returns None on
/// any malformed input; callers should treat None as "skip this stop".
pub fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let mut out = [0u8; 3];
            for (i, ch) in hex.chars().enumerate() {
                let v = ch.to_digit(16)? as u8;
                out[i] = v * 17;
            }
            Some((out[0], out[1], out[2]))
        }
        _ => None,
    }
}
