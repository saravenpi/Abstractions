//! The script and the timing of the wake-up sequence.

use super::loading::Arrival;

/// Seconds before the first line, and between two lines.
const PACE: (f32, f32) = (0.3, 0.8);

/// Seconds between a line and its status word.
pub const STATUS_DELAY: f32 = 0.45;

/// Seconds the finished sequence stays before the fade.
const HOLD: f32 = 0.65;

/// Seconds the fade to the world takes.
pub const FADE: f32 = 0.8;

/// Returns the lines of the sequence as label and status: the full wake-up of a new world, or
/// the single line of a resumed one.
pub fn script(arrival: &Arrival) -> Vec<(&'static str, String)> {
    if !arrival.fresh {
        return vec![("LINK", "RESTORED".to_string())];
    }
    let place = format!("{}, {}", arrival.planet, arrival.star).to_uppercase();
    vec![
        ("QUANTUM LINK", "ESTABLISHED".to_string()),
        ("OPERATOR", "CONNECTED".to_string()),
        ("UNIT 01", "ONLINE".to_string()),
        ("COMPANION UNIT", "ONLINE".to_string()),
        ("LOCATION", place),
    ]
}

/// Returns the second at which line `index` appears.
pub fn line_time(index: usize) -> f32 {
    PACE.0 + PACE.1 * index as f32
}

/// Returns the second at which a sequence of `lines` lines starts to fade, and the second it ends.
pub fn timeline(lines: usize) -> (f32, f32) {
    let fade_at = line_time(lines.saturating_sub(1)) + STATUS_DELAY + HOLD;
    (fade_at, fade_at + FADE)
}
