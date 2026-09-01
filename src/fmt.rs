//! Human-readable formatting shared by modules.

/// `3d4h` / `2h14` / `18m` / `45s`.
pub fn duration(secs: u64) -> String {
    match secs {
        s if s >= 86_400 => format!("{}d{}h", s / 86_400, (s % 86_400) / 3_600),
        s if s >= 3_600 => format!("{}h{:02}", s / 3_600, (s % 3_600) / 60),
        s if s >= 60 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}

/// `630k` / `1M` / `1.5M`.
pub fn tokens(n: u64) -> String {
    match n {
        n if n >= 1_000_000 && n % 1_000_000 == 0 => format!("{}M", n / 1_000_000),
        n if n >= 1_000_000 => format!("{}.{}M", n / 1_000_000, (n % 1_000_000) / 100_000),
        n if n >= 1_000 => format!("{}k", n / 1_000),
        n => n.to_string(),
    }
}
