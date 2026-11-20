//! Air: how much of it a planet keeps, what it is made of, how high it reaches, how much it
//! warms the ground and when water boils under it.

mod born;
mod column;
mod gas;
mod mix;
mod tune;

#[cfg(test)]
mod tests_column;
#[cfg(test)]
mod tests_mix;

pub use born::{born, escape};
pub use column::{dense, heights};
pub use gas::Gas;
pub use mix::{Mix, Part};
pub use tune::TOP_SCALES;

/// Pressure, the air of the Earth being 1, under which a planet has no air to speak of.
pub const NO_AIR: f64 = 0.01;

/// Highest pressure a planet keeps, in Earth atmospheres.
pub const PRESSURE_CAP: f64 = 5.0;

/// Pressure under which water has no liquid state, in Earth atmospheres: its triple point.
pub const TRIPLE_POINT: f64 = 0.006;

/// Leanest and richest stock of gas a planet is born with, in Earth atmospheres.
const VOLATILES: (f64, f64) = (0.03, 30.0);

/// Grip at which a planet keeps half of its gas, see [`pressure`].
const HALF_GRIP: f64 = 1.2;

/// Flux from which bare rock melts: 1,000 K on a black sphere that sends back a fifth of the light.
const MELTING: f64 = 208.0;

/// Pressure of the rock vapour over a world hot enough to melt, in Earth atmospheres.
const ROCK_VAPOUR: f64 = 0.02;

/// Warming in kelvin by the air of the Earth.
const EARTH_GREENHOUSE: f64 = 33.0;

/// Kelvin at which water boils under the air of the Earth.
const EARTH_BOILING: f64 = 373.15;

/// Gas constant over the heat of vaporisation of water, per kelvin: 8.314 over 40,660 J/mol.
const VAPOUR: f64 = 2.045e-4;

/// Returns the air pressure at sea level of a planet, the air of the Earth being 1, from 0 to 5.
///
/// `gravity` and `size` are the surface gravity and the radius over those of the Earth.
/// `stock` from 0 to 1 places the gas the planet was born with between 0.03 and 30 Earth
/// atmospheres on a log scale, and a stronger gravity packs the same gas tighter. The planet
/// keeps a share of it that follows its grip: the escape speed, the square root of gravity
/// times radius, over the fifth root of the flux, since heat and a close star both blow gas
/// away. The share is `1 / (1 + (1.2 / grip)^4)`: nearly all above a grip of 2.6, half at 1.2
/// and under a fiftieth at the 0.45 of Mars. A world hot enough to melt its rock always keeps
/// a thin air of rock vapour.
pub fn pressure(stock: f64, gravity: f64, size: f64, flux: f64) -> f64 {
    let born = VOLATILES.0 * (VOLATILES.1 / VOLATILES.0).powf(stock);
    let grip = (gravity * size).sqrt() / flux.powf(0.2);
    let kept = 1.0 / (1.0 + (HALF_GRIP / grip).powi(4));
    let vapour = if flux >= MELTING { ROCK_VAPOUR } else { 0.0 };
    (born * gravity * kept).clamp(vapour, PRESSURE_CAP)
}

/// Returns the warming in kelvin by air of a given pressure with a steady mix of gases.
///
/// It grows as the square root of the pressure: 33 K under the air of the Earth, 74 K under 5.
pub fn greenhouse(pressure: f64) -> f64 {
    EARTH_GREENHOUSE * pressure.sqrt()
}

/// Returns the kelvin at which water boils under a given pressure, above the triple point.
///
/// This is the Clausius-Clapeyron relation from the 373 K of the Earth: 268 K at the triple
/// point pressure, 425 K under 5 Earth atmospheres.
pub fn boiling(pressure: f64) -> f64 {
    1.0 / (1.0 / EARTH_BOILING - VAPOUR * pressure.ln())
}
