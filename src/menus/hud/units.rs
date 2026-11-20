//! Units of the flight instruments: heights, speeds and levers as short readable text.

use crate::ui::units::distance;
pub use crate::ui::units::speed;

/// Formats a height in metres as a distance. A height below the reference keeps its sign.
pub fn height(metres: f64) -> String {
    let shown = distance(metres.abs());
    match metres < 0.0 && shown != "0 m" {
        true => format!("-{shown}"),
        false => shown,
    }
}

/// Formats a fraction as a whole percentage: 0.654 reads "65 %".
pub fn percent(fraction: f64) -> String {
    format!("{:.0} %", fraction * 100.0)
}
