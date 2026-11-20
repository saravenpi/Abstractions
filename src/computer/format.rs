//! Short numbers and clipped words for the computer tables.

/// Formats a number with three significant digits under 100 and no decimals above.
pub(super) fn short(value: f64) -> String {
    match value {
        v if v < 9.995 => format!("{v:.2}"),
        v if v < 99.95 => format!("{v:.1}"),
        v => format!("{v:.0}"),
    }
}

/// Returns `text` cut to at most `chars` characters, ending in `..` when it was cut.
pub fn clip(text: &str, chars: usize) -> String {
    if text.chars().count() <= chars {
        return text.to_string();
    }
    let kept: String = text.chars().take(chars.saturating_sub(2)).collect();
    format!("{kept}..")
}
