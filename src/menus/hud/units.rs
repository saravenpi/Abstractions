//! Units of the flight instruments: heights, speeds and levers as short readable text.

use crate::computer::distance;

/// Formats a height in metres as m, km, Mm or gly. A height below the reference keeps its sign.
pub fn height(metres: f64) -> String {
    let shown = distance(metres.abs());
    match metres < 0.0 && shown != "0 m" {
        true => format!("-{shown}"),
        false => shown,
    }
}

/// Formats a speed in metres per second as m/s, km/s, Mm/s or gly/s.
pub fn speed(metres_per_second: f64) -> String {
    format!("{}/s", distance(metres_per_second.abs()))
}

/// Formats a fraction as a whole percentage: 0.654 reads "65 %", reverse thrust reads "-50 %".
pub fn percent(fraction: f64) -> String {
    format!("{:.0} %", fraction * 100.0)
}
