//! The air a body flies through: dynamic pressure, the speed of sound and the Mach number.

use crate::universe::{GAS_CONSTANT, Mix};

/// Returns the dynamic pressure in pascals of air of `density` kilograms per cubic metre meeting a
/// body at `speed` metres per second: `1/2 rho v^2`. It is zero without air, whatever the speed.
pub fn dynamic_pressure(density: f32, speed: f32) -> f32 {
    0.5 * density.max(0.0) * speed * speed
}

/// Returns the speed of sound in metres per second in a gas `mix` at `temperature` kelvin:
/// `sqrt(gamma R T / M)`, with the ratio of heats `gamma = 1 / (1 - kappa)`. It is 340 m/s for the
/// air of the home planet at 288 K.
pub fn sound_speed(temperature: f64, mix: &Mix) -> f32 {
    let gamma = 1.0 / (1.0 - mix.kappa());
    (gamma * GAS_CONSTANT * temperature.max(0.0) / mix.molar()).sqrt() as f32
}

/// Returns the temperature in kelvin of the air `height` metres over the ground, for air at
/// `surface` kelvin whose scale height is `scale` metres: rising air cools as `T (p / p0)^kappa`,
/// and the pressure falls by a factor e over a scale height. The cooling stops at `limit` metres,
/// the top of the weather layer, above which the air keeps its temperature.
pub fn temperature_at(surface: f64, kappa: f64, (scale, limit): (f64, f64), height: f64) -> f64 {
    surface * (-kappa * height.clamp(0.0, limit) / scale).exp()
}

/// Returns the Mach number of a body at `speed` metres per second in air where sound travels at
/// `sound`, zero when there is no sound speed to measure against.
pub fn mach(speed: f32, sound: f32) -> f32 {
    if sound > 0.0 { speed / sound } else { 0.0 }
}
