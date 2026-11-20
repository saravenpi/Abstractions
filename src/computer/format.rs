//! Human readable distances and gravity for the computer tables.

use crate::universe::{LY, STANDARD_GRAVITY};

/// Metres in one kilometre and in one megametre, with their unit names.
const UNITS: [(f64, &str); 2] = [(1e3, "km"), (1e6, "Mm")];

/// Largest number shown in a unit before the next unit takes over.
const UNIT_LIMIT: f64 = 999.5;

/// Formats a number with three significant digits under 100 and no decimals above.
pub(super) fn short(value: f64) -> String {
    match value {
        v if v < 9.995 => format!("{v:.2}"),
        v if v < 99.95 => format!("{v:.1}"),
        v => format!("{v:.0}"),
    }
}

/// Formats a distance in metres as m, km, Mm or light-years, whichever keeps the number under 1000.
///
/// Past 999 Mm the unit is the light-year, however large the number gets.
pub fn distance(metres: f64) -> String {
    let metres = metres.max(0.0);
    if metres < UNIT_LIMIT {
        return format!("{metres:.0} m");
    }
    match UNITS.iter().find(|(scale, _)| metres / scale < UNIT_LIMIT) {
        Some((scale, unit)) => format!("{} {unit}", short(metres / scale)),
        None => format!("{} ly", short(metres / LY)),
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

/// Formats a surface gravity in metres per second squared as a multiple of the home planet's.
pub fn gravity(acceleration: f64) -> String {
    format!("{:.2} g", acceleration / STANDARD_GRAVITY)
}
