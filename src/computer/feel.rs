//! What the numbers of a planet mean for the player: bars, kilometres, comparisons with home,
//! and a plain word for the pull, the air, the heat and the length of the day.

use super::words::{AIR, FREEZING};

/// What the air does for a glider, from none to thick, in the order of [`AIR`] and one more.
const LIFT: [&str; 4] = [
    "No lift: boots only",
    "Weak lift",
    "Wings work",
    "Strong lift, hard on the wings",
];

/// Returns what the air of a planet with `pressure` does for the wings of the player.
pub fn lift(pressure: f64) -> &'static str {
    let at = AIR
        .iter()
        .take_while(|(limit, _)| pressure >= *limit)
        .count();
    LIFT[at]
}

/// Returns what a gravity `ratio` times that of home means for a player on foot.
pub fn gravity_feel(ratio: f64) -> &'static str {
    match ratio {
        r if r < 0.3 => "Very light: huge jumps",
        r if r < 0.8 => "Light",
        r if r < 1.25 => "Like home",
        r if r < 2.0 => "Heavy: short jumps",
        _ => "Crushing",
    }
}

/// Returns a word for a temperature in kelvin: freezing, cold, mild, hot or scorching.
pub fn heat(kelvin: f64) -> &'static str {
    match kelvin - FREEZING {
        c if c < -100.0 => "Freezing",
        c if c < 0.0 => "Cold",
        c if c < 35.0 => "Mild",
        c if c < 100.0 => "Hot",
        _ => "Scorching",
    }
}

/// Returns what a day `ratio` times as long as the day of home means.
pub fn day_feel(ratio: f64) -> &'static str {
    match ratio {
        r if r < 0.5 => "Short days: the sun moves fast",
        r if r < 2.0 => "Days like home",
        _ => "Long days: nights last for hours",
    }
}

/// Formats `value` against `home` as a multiple with one decimal, such as `1.2 x home`, and two
/// decimals under a tenth so that a small ratio does not read as zero. Nothing without a ratio.
pub fn versus(value: f64, home: f64) -> String {
    let ratio = (value / home).max(0.0);
    if !ratio.is_finite() {
        return String::new();
    }
    match ratio {
        r if r < 0.1 => format!("{r:.2} x home"),
        r if r < 100.0 => format!("{r:.1} x home"),
        r => format!("{r:.0} x home"),
    }
}
