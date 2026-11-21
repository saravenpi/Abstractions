//! The one place the interface turns a quantity into words: distances from one metre to
//! billions of light-years, speeds, times, masses, gravity and air. Every number has its unit
//! next to it, with three significant digits, a space between thousands and no scientific
//! notation. No other file of the interface writes a unit by hand.

mod distance;
mod measure;
mod number;
mod planet;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_rules;
#[cfg(test)]
mod tests_zero;

pub use distance::{
    AU_NOTE, LIGHT_YEAR_NOTE, SHORT_NOTE, distance, distance_short, distances, distances_short,
};
pub use measure::{duration, speed};
pub use number::{fixed, plain, spaced};
pub use planet::{
    gravity, gravity_ratio, gravity_si, mass, pressure, pressure_bar, pressure_ratio,
};
