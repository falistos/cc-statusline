//! Parser for style strings, mapping to anstyle.
//!
//! Examples:
//!   - `"bold red"`               bold + red fg
//!   - `"red on blue"`            red fg, blue bg
//!   - `"#ff8800"`                truecolor fg
//!   - `"bg:#222 fg:white dim"`   explicit channels + modifier
//!   - `"244"`                    ANSI 256 fg

use crate::render::term::TermCaps;
use anstyle::{Ansi256Color, AnsiColor, Color, RgbColor, Style as AnstyleStyle};
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fg: Option<Col>,
    pub bg: Option<Col>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Col {
    Named(AnsiColor),
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Error)]
pub enum StyleError {
    #[error("unknown style token: {0}")]
    UnknownToken(String),
    #[error("dangling 'on' (expected a background color after it)")]
    DanglingOn,
}

impl Style {
    pub fn parse(s: &str) -> Result<Self, StyleError> {
        let mut style = Style::default();
        let tokens: Vec<&str> = s.split_whitespace().collect();
        let mut i = 0;
        while i < tokens.len() {
            let tok = tokens[i];
            let lower = tok.to_lowercase();
            match lower.as_str() {
                "bold" => style.bold = true,
                "italic" => style.italic = true,
                "dim" | "dimmed" => style.dim = true,
                "underline" => style.underline = true,
                "none" | "default" => {}
                "on" => {
                    i += 1;
                    let next = tokens.get(i).ok_or(StyleError::DanglingOn)?;
                    style.bg = parse_color(next)?
                        .ok_or_else(|| StyleError::UnknownToken(next.to_string()))?
                        .into();
                }
                _ => {
                    if let Some(rest) = lower.strip_prefix("fg:") {
                        style.fg = parse_color(rest)?;
                    } else if let Some(rest) = lower.strip_prefix("bg:") {
                        style.bg = parse_color(rest)?;
                    } else if let Some(c) = parse_color(tok)? {
                        if style.fg.is_none() {
                            style.fg = Some(c);
                        } else {
                            style.bg = Some(c);
                        }
                    } else {
                        return Err(StyleError::UnknownToken(tok.to_string()));
                    }
                }
            }
            i += 1;
        }
        Ok(style)
    }

    pub fn to_anstyle(&self, term: &TermCaps) -> AnstyleStyle {
        let mut s = AnstyleStyle::new();
        if self.bold {
            s = s.bold();
        }
        if self.italic {
            s = s.italic();
        }
        if self.dim {
            s = s.dimmed();
        }
        if self.underline {
            s = s.underline();
        }
        if let Some(fg) = &self.fg {
            s = s.fg_color(Some(col_to_anstyle(fg, term)));
        }
        if let Some(bg) = &self.bg {
            s = s.bg_color(Some(col_to_anstyle(bg, term)));
        }
        s
    }
}

fn col_to_anstyle(c: &Col, term: &TermCaps) -> Color {
    match c {
        Col::Named(n) => Color::Ansi(*n),
        Col::Ansi256(n) => {
            if term.supports_256() {
                Color::Ansi256(Ansi256Color(*n))
            } else {
                Color::Ansi(ansi256_fallback(*n))
            }
        }
        Col::Rgb(r, g, b) => {
            if term.supports_truecolor() {
                Color::Rgb(RgbColor(*r, *g, *b))
            } else if term.supports_256() {
                Color::Ansi256(Ansi256Color(rgb_to_ansi256(*r, *g, *b)))
            } else {
                Color::Ansi(rgb_to_ansi16(*r, *g, *b))
            }
        }
    }
}

fn parse_color(s: &str) -> Result<Option<Col>, StyleError> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex).map(Some);
    }
    if let Ok(n) = s.parse::<u8>() {
        return Ok(Some(Col::Ansi256(n)));
    }
    if let Some(named) = parse_named(&s.to_lowercase()) {
        return Ok(Some(Col::Named(named)));
    }
    Ok(None)
}

fn parse_hex(hex: &str) -> Result<Col, StyleError> {
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| StyleError::UnknownToken(format!("#{hex}")))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| StyleError::UnknownToken(format!("#{hex}")))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| StyleError::UnknownToken(format!("#{hex}")))?;
        Ok(Col::Rgb(r, g, b))
    } else if hex.len() == 3 {
        // #rgb shorthand → expand to #rrggbb
        let mut bytes = [0u8; 3];
        for (i, ch) in hex.chars().enumerate() {
            let v = ch
                .to_digit(16)
                .ok_or_else(|| StyleError::UnknownToken(format!("#{hex}")))?
                as u8;
            bytes[i] = v * 17;
        }
        Ok(Col::Rgb(bytes[0], bytes[1], bytes[2]))
    } else {
        Err(StyleError::UnknownToken(format!("#{hex}")))
    }
}

fn parse_named(s: &str) -> Option<AnsiColor> {
    Some(match s {
        "black" => AnsiColor::Black,
        "red" => AnsiColor::Red,
        "green" => AnsiColor::Green,
        "yellow" => AnsiColor::Yellow,
        "blue" => AnsiColor::Blue,
        "magenta" | "purple" => AnsiColor::Magenta,
        "cyan" => AnsiColor::Cyan,
        "white" => AnsiColor::White,
        "bright-black" | "gray" | "grey" => AnsiColor::BrightBlack,
        "bright-red" => AnsiColor::BrightRed,
        "bright-green" => AnsiColor::BrightGreen,
        "bright-yellow" => AnsiColor::BrightYellow,
        "bright-blue" => AnsiColor::BrightBlue,
        "bright-magenta" | "bright-purple" => AnsiColor::BrightMagenta,
        "bright-cyan" => AnsiColor::BrightCyan,
        "bright-white" => AnsiColor::BrightWhite,
        _ => return None,
    })
}

/// Crude ANSI 256 → ANSI 16 fallback.
fn ansi256_fallback(n: u8) -> AnsiColor {
    match n {
        0 => AnsiColor::Black,
        1 => AnsiColor::Red,
        2 => AnsiColor::Green,
        3 => AnsiColor::Yellow,
        4 => AnsiColor::Blue,
        5 => AnsiColor::Magenta,
        6 => AnsiColor::Cyan,
        7 => AnsiColor::White,
        8 => AnsiColor::BrightBlack,
        9 => AnsiColor::BrightRed,
        10 => AnsiColor::BrightGreen,
        11 => AnsiColor::BrightYellow,
        12 => AnsiColor::BrightBlue,
        13 => AnsiColor::BrightMagenta,
        14 => AnsiColor::BrightCyan,
        15 => AnsiColor::BrightWhite,
        232..=255 => AnsiColor::BrightBlack,
        _ => AnsiColor::White,
    }
}

/// RGB → ANSI 256 via the standard 6x6x6 color cube.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let r6 = (r as u16 * 5 / 255) as u8;
    let g6 = (g as u16 * 5 / 255) as u8;
    let b6 = (b as u16 * 5 / 255) as u8;
    16 + 36 * r6 + 6 * g6 + b6
}

/// RGB → nearest of the 16 ANSI colors (coarse fallback).
fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> AnsiColor {
    let bright = r > 170 || g > 170 || b > 170;
    let rh = r > 96;
    let gh = g > 96;
    let bh = b > 96;
    match (rh, gh, bh, bright) {
        (false, false, false, _) => AnsiColor::Black,
        (true, false, false, true) => AnsiColor::BrightRed,
        (true, false, false, false) => AnsiColor::Red,
        (false, true, false, true) => AnsiColor::BrightGreen,
        (false, true, false, false) => AnsiColor::Green,
        (true, true, false, true) => AnsiColor::BrightYellow,
        (true, true, false, false) => AnsiColor::Yellow,
        (false, false, true, true) => AnsiColor::BrightBlue,
        (false, false, true, false) => AnsiColor::Blue,
        (true, false, true, true) => AnsiColor::BrightMagenta,
        (true, false, true, false) => AnsiColor::Magenta,
        (false, true, true, true) => AnsiColor::BrightCyan,
        (false, true, true, false) => AnsiColor::Cyan,
        (true, true, true, true) => AnsiColor::BrightWhite,
        (true, true, true, false) => AnsiColor::White,
    }
}
