//! The physics the sound of a ship follows, as pure functions: where the spectrum of a jet
//! peaks, how loud it is for its thrust, which way it radiates, and what distance and air do
//! to it on the way to the listener.

use std::f32::consts::PI;

use super::tune::{
    EARSHOT, GROUND_REACH, HELMHOLTZ, LOBE, LOBE_FLOOR, NEAR, SEA_AIR, THROTTLE_FLOOR,
};

/// Returns the frequency in hertz at which the mixing noise of a supersonic jet peaks, for a
/// nozzle of `diameter` metres at a thrust `level` in air where sound travels at `sound` metres
/// per second.
///
/// The peak of a hot supersonic jet sits at a fixed Helmholtz number `f D / c` of the fully
/// expanded jet. The mass flow follows the thrust, so the jet is `sqrt(level)` as wide and a
/// throttled engine sounds higher and thinner.
pub fn peak_hz(sound: f32, diameter: f32, level: f32) -> f32 {
    HELMHOLTZ * sound / (diameter * level.clamp(THROTTLE_FLOOR, 2.0).sqrt())
}

/// Returns the sound pressure of a jet at a thrust `level`, 1 at full throttle.
///
/// A rocket exhaust turns a fixed share of its mechanical power into sound. At a fixed exhaust
/// speed that power follows the thrust, and the pressure is its square root.
pub fn loudness(level: f32) -> f32 {
    level.max(0.0).sqrt()
}

/// Returns how much of the mixing noise of a jet goes toward a listener, from [`LOBE_FLOOR`]
/// to 1, for the cosine `aspect` of the angle between the exhaust and the listener.
///
/// The large turbulent structures radiate as Mach waves into a cone around the exhaust: the
/// noise is loudest at an angle behind the nozzle, weaker straight behind it and weakest ahead.
pub fn lobe(aspect: f32) -> f32 {
    let off = (aspect.clamp(-1.0, 1.0) - LOBE.0) / LOBE.1;
    LOBE_FLOOR + (1.0 - LOBE_FLOOR) * (-off * off).exp()
}

/// Returns the pressure of a sound `distance` metres away against what it is at the reference
/// distance: it spreads over a sphere and falls as `1 / r`, and is gone at the edge of earshot.
pub fn spread(distance: f32) -> f32 {
    let edge = 1.0 - (distance / EARSHOT).clamp(0.0, 1.0).powi(4);
    NEAR.1 / distance.max(NEAR.0) * edge
}

/// Returns the gain that brings white noise through a band-pass at `hz` of resonance `q` to a
/// level of 1, at a sample rate `rate`: the band lets through `pi hz / (3 q rate)` of the power.
pub fn band_gain(hz: f32, q: f32, rate: f32) -> f32 {
    (3.0 * q * rate / (PI * hz.max(1.0))).sqrt()
}

/// Returns how close the ground is to the lift nozzles of a ship `height` metres above it,
/// from 0 (out of reach of the plumes) to 1 (on the ground).
pub fn ground(height: f32) -> f32 {
    let near = (1.0 - height / GROUND_REACH).clamp(0.0, 1.0);
    near * near
}

/// Returns the dynamic pressure in pascals on a ship at `speed` metres per second in air of
/// density `air`, a share of the air at sea level at home.
pub fn pressure(air: f32, speed: f32) -> f32 {
    crate::speed::dynamic_pressure(air * SEA_AIR, speed)
}
