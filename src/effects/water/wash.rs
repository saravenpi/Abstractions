//! How hard a source blows on the sea: the wash of its push, felt at the surface under it.
//!
//! A source that carries its weight on the air, a wing or an engine, drives that much momentum
//! down each second. As an actuator disc of its own size it throws the air down at the far wash
//! speed; the flow spreads as a jet does, its speed falling as one over the distance once past
//! the size of the source. The sea feels the dynamic pressure of that flow plus a share of the
//! flight of the source itself, so both a low hover and a fast low pass tear spray off it.

use std::f32::consts::{PI, TAU};

use super::tune::{
    BOW_SHARE, KELVIN, MAX_THROWN, RIPPLE_PRESSURE, RIPPLE_WAVELENGTH, SKIM_DEPTH, SKIM_SPRAY,
    SPRAY_PRESSURE,
};
use crate::render::smoothstep;

/// Returns the far wash speed in metres per second of a source `size` metres across that pushes
/// `push` newtons on air of `density` kilograms per cubic metre while moving at `speed`.
///
/// Momentum theory: the induced speed `w` solves `push = 2 rho A (v + w) w`, and the far wake
/// moves at twice it. A fast source meets more air each second and throws it down slower.
pub fn far_wash(push: f32, size: f32, density: f32, speed: f32) -> f32 {
    let area = PI * size * size * 0.25;
    if push <= 0.0 || density <= 0.0 || area <= 0.0 {
        return 0.0;
    }
    let v = speed.max(0.0);
    (v * v + 2.0 * push / (density * area)).sqrt() - v
}

/// Returns the dynamic pressure in pascals the flow of a source puts on the sea `height` metres
/// under it, from its far wash `wash` and its `speed`, in air of `density`.
pub fn surface_pressure(wash: f32, speed: f32, size: f32, density: f32, height: f32) -> f32 {
    let reach = size / (size + height.max(0.0)).max(1e-3);
    let flow = wash * wash + BOW_SHARE * speed * speed;
    0.5 * density.max(0.0) * flow * reach * reach
}

/// Returns how strongly a surface `pressure` tears spray, from 0 under [`SPRAY_PRESSURE`] toward
/// 1 far above it.
pub fn spray_strength(pressure: f32) -> f32 {
    match pressure > SPRAY_PRESSURE {
        true => 1.0 - SPRAY_PRESSURE / pressure,
        false => 0.0,
    }
}

/// Returns how strongly a source `size` metres across, `height` metres over the sea and moving
/// at `speed` along it skims the surface and tears spray by flying fast and low: 0 when it is
/// higher than its own size or deeper than [`SKIM_DEPTH`], else rising with the speed.
pub fn skim(speed: f32, height: f32, size: f32) -> f32 {
    match height <= size && height >= -SKIM_DEPTH {
        true => smoothstep(SKIM_SPRAY.0, SKIM_SPRAY.1, speed),
        false => 0.0,
    }
}

/// Returns the speed in metres per second at which ripples run out over a sea under `gravity`:
/// the phase speed of deep water waves `RIPPLE_WAVELENGTH` long.
pub fn ripple_speed(gravity: f32) -> f32 {
    (gravity.max(0.0) * RIPPLE_WAVELENGTH / TAU).sqrt()
}

/// Returns the rate in metres per second at which the half width of the wake of a source
/// moving at `speed` grows: the Kelvin wedge.
pub fn kelvin_spread(speed: f32) -> f32 {
    speed.max(0.0) * KELVIN.tan()
}

/// Adds `amount` to the running count `due` and returns how many whole drops to throw now, at
/// most [`MAX_THROWN`]. What a frame cannot throw is dropped, never carried into a burst.
pub fn thrown(due: &mut f32, amount: f32) -> u32 {
    *due += amount.max(0.0);
    let whole = due.floor();
    *due -= whole;
    (whole as u32).min(MAX_THROWN)
}

/// What the flow of a source does to the sea under it this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Flow {
    /// Speed in metres per second of the flow over the surface.
    pub surface: f32,
    /// How strongly it tears spray, from 0 to 1.
    pub spray: f32,
    /// How strongly it rings the surface with ripples, from 0 to 1.
    pub ripple: f32,
}

/// Returns the flow on the sea `height` metres under a source `size` metres across that pushes
/// `push` newtons on air of `density` while moving at `speed`. Nothing reaches the sea from
/// under it.
pub fn flow(push: f32, size: f32, density: f32, (speed, height): (f32, f32)) -> Flow {
    if height < 0.0 || density <= 0.0 {
        return Flow::default();
    }
    let wash = far_wash(push, size, density, speed);
    let pressure = surface_pressure(wash, speed, size, density, height);
    Flow {
        surface: (2.0 * pressure / density).sqrt(),
        spray: spray_strength(pressure),
        ripple: (1.0 - RIPPLE_PRESSURE / pressure.max(1e-6)).max(0.0),
    }
}
