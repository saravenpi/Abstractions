//! Distances from one metre to billions of light-years, in the unit a person would pick.

use super::number::plain;
use crate::universe::{AU, LY};

/// A unit of distance: the smallest distance in metres that uses it, the metres in one, its
/// name and its short name.
type Unit = (f64, f64, &'static str, &'static str);

/// Metres to the first light-year row: a tenth of a light-year, far past a thousand AU.
const LY_FROM: f64 = 0.1 * LY;

/// The units, from the smallest. Kilometres give way to millions of kilometres at 999 999 km,
/// those to AU at 10 million km, AU to light-years at a tenth of one.
pub const UNITS: [Unit; 7] = [
    (0.0, 1.0, "m", "m"),
    (999.5, 1e3, "km", "km"),
    (9.995e8, 1e9, "million km", "M km"),
    (1e10, AU, "AU", "AU"),
    (LY_FROM, LY, "light-years", "ly"),
    (
        1e6 * LY,
        1e6 * LY,
        "million light-years",
        "M ly",
    ),
    (
        1e9 * LY,
        1e9 * LY,
        "billion light-years",
        "G ly",
    ),
];

/// What 1 AU means, for a legend or a tooltip.
pub const AU_NOTE: &str =
    "1 AU = the distance from a planet like home to its star, about 150 million km";

/// What a light-year means, for a legend or a tooltip.
pub const LIGHT_YEAR_NOTE: &str = "1 light-year = the distance light travels in a year";

/// Returns the row of [`UNITS`] that suits a distance of `metres`.
fn row(metres: f64) -> usize {
    UNITS.iter().rposition(|unit| unit.0 <= metres).unwrap_or(0)
}

/// Returns `metres` in the unit of row `at`, with its name, short when `short`.
fn write(metres: f64, at: usize, short: bool) -> String {
    let (_, scale, long, brief) = UNITS[at];
    let value = metres / scale;
    let name = match (at, value <= 1.0) {
        (4, true) => "light-year",
        _ => long,
    };
    format!("{} {}", plain(value), [name, brief][usize::from(short)])
}

/// Returns a distance in words: "340 m", "12 400 km", "3.2 million km", "1.5 AU",
/// "4.2 light-years", "2.5 million light-years".
pub fn distance(metres: f64) -> String {
    let metres = metres.max(0.0);
    write(metres, row(metres), false)
}

/// Returns a distance like [`distance`] with "ly" for light-years, where space is short.
pub fn distance_short(metres: f64) -> String {
    let metres = metres.max(0.0);
    write(metres, row(metres), true)
}

/// Returns the distances of one list like [`distances`], with the short unit names.
pub fn distances_short(metres: &[f64]) -> Vec<String> {
    let largest = metres.iter().copied().fold(0.0, f64::max);
    let at = row(largest);
    metres.iter().map(|m| write(m.max(0.0), at, true)).collect()
}

/// What the short unit names mean, for a tooltip over a list that uses them.
pub const SHORT_NOTE: &str =
    "ly = light-year, what light travels in a year. AU = from a planet like home to its star";

/// Returns the distances of one list in one unit, the one the largest needs, so rows compare.
pub fn distances(metres: &[f64]) -> Vec<String> {
    let largest = metres.iter().copied().fold(0.0, f64::max);
    let at = row(largest);
    metres
        .iter()
        .map(|m| write(m.max(0.0), at, false))
        .collect()
}
