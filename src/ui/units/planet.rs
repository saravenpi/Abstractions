//! The quantities of a planet: mass, pull and air.

use super::number::plain;
use crate::universe::STANDARD_GRAVITY;

/// Returns a mass: "300 g", "12 kg" or "1.5 t".
pub fn mass(kg: f64) -> String {
    match kg.max(0.0) {
        k if k < 1.0 => format!("{} g", plain(k * 1e3)),
        k if k < 999.5 => format!("{} kg", plain(k)),
        k => format!("{} t", plain(k / 1e3)),
    }
}

/// Returns a pull in metres per second squared as a multiple of home: "1.2 x home".
pub fn gravity_ratio(acceleration: f64) -> String {
    format!("{} x home", plain(acceleration / STANDARD_GRAVITY))
}

/// Returns a pull in the SI unit: "11.8 m/s2".
pub fn gravity_si(acceleration: f64) -> String {
    format!("{acceleration:.1} m/s2")
}

/// Returns a pull as a multiple of home with the SI value beside it: "1.2 x home, 11.8 m/s2".
pub fn gravity(acceleration: f64) -> String {
    format!(
        "{}, {}",
        gravity_ratio(acceleration),
        gravity_si(acceleration)
    )
}

/// Returns an air pressure, home having 1, as a multiple of home: "0.6 x home".
pub fn pressure_ratio(home_multiple: f64) -> String {
    format!("{} x home", plain(home_multiple))
}

/// Returns an air pressure, home having 1 bar, in bar: "0.6 bar".
pub fn pressure_bar(home_multiple: f64) -> String {
    format!("{} bar", plain(home_multiple))
}

/// Returns an air pressure as a multiple of home with bar beside it: "0.6 x home, 0.6 bar".
pub fn pressure(home_multiple: f64) -> String {
    format!(
        "{}, {}",
        pressure_ratio(home_multiple),
        pressure_bar(home_multiple)
    )
}
