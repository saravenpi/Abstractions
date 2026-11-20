//! The physics of a wake: the angle of its arms, the wavelength of its waves and the height the
//! push of a source raises.
//!
//! A slow source leaves the Kelvin wedge of half angle 19.47 degrees. A fast one leaves narrower
//! arms as its hull Froude number grows (Rabaud and Moisy 2013, "Ship wakes: Kelvin or Mach
//! angle?", <https://doi.org/10.1103/PhysRevLett.110.214503>).

use std::f32::consts::TAU;

use super::tune::{ARM_FLOOR, GAIN, KELVIN, MAX_AMPLITUDE, TOUCH_SHARE, TRANSVERSE};
use crate::render::smoothstep;

/// Returns the hull Froude number `v / sqrt(g L)` of a source `size` metres across moving at
/// `speed` metres per second under `gravity`.
pub fn froude(speed: f32, gravity: f32, size: f32) -> f32 {
    speed.abs() / (gravity * size).max(1e-6).sqrt()
}

/// Returns the half angle in radians of the wedge behind a source `size` metres across moving at
/// `speed` metres per second under `gravity`.
///
/// It is the Kelvin angle for a slow source and `atan(1 / (2 sqrt(2 pi) Fr))` once that is
/// narrower, from a Froude number of about one half (Rabaud and Moisy 2013,
/// <https://doi.org/10.1103/PhysRevLett.110.214503>).
pub fn wedge(speed: f32, gravity: f32, size: f32) -> f32 {
    let fr = froude(speed, gravity, size).max(1e-6);
    (1.0 / (2.0 * TAU.sqrt() * fr)).atan().min(KELVIN)
}

/// Returns the half angle in radians the arms of a wake are laid at: the [`wedge`], never
/// narrower than [`ARM_FLOOR`].
pub fn arm_angle(speed: f32, gravity: f32, size: f32) -> f32 {
    wedge(speed, gravity, size).max(ARM_FLOOR)
}

/// Returns the wavelength in metres of the transverse waves behind a source moving at `speed`
/// metres per second under `gravity`: the waves whose phase speed is the speed of the source,
/// `2 pi v^2 / g`.
pub fn transverse_wavelength(speed: f32, gravity: f32) -> f32 {
    TAU * speed * speed / gravity.max(1e-6)
}

/// Returns how much of the waves of a source with Froude number `fr` are transverse, from 1 for a
/// slow one to 0 for a fast one.
pub fn transverse_share(fr: f32) -> f32 {
    1.0 - smoothstep(TRANSVERSE.0, TRANSVERSE.1, fr)
}

/// Returns the height in metres of the waves a source raises, from the dynamic `pressure` in
/// pascals of its air flow on a liquid of `density` kilograms per cubic metre under `gravity`,
/// and from its `speed` when it `touches` the surface.
///
/// The flow piles up a head of liquid `p / (rho g)`, scaled by [`GAIN`]. A source that touches
/// the surface also shoves the liquid aside with a share of its own head `v^2 / 2g`.
pub fn amplitude(pressure: f32, density: f32, gravity: f32, (speed, touches): (f32, bool)) -> f32 {
    let weight = (density * gravity).max(1e-6);
    let hull =
        f32::from(u8::from(touches)) * TOUCH_SHARE * speed * speed / (2.0 * gravity.max(1e-6));
    (GAIN * pressure.max(0.0) / weight + hull).min(MAX_AMPLITUDE)
}
