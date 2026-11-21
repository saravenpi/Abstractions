//! Units of the flight instruments: heights, speeds and levers as short readable text.

pub use crate::ui::units::speed;
use crate::ui::units::{distance, fixed};

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
    format!("{} %", fixed(fraction * 100.0, 0))
}
