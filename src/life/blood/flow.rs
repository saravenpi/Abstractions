//! The physics of blood: how much a body holds, how hard it is pushed, and how it leaves a wound.

use std::f32::consts::PI;

use super::tune::{
    BASE, CLOT, CLOT_MAJOR, DENSITY, DISCHARGE, LIFT, OPEN, PORES, SHARE, VENOUS, VESSEL,
    VESSEL_EXPONENT,
};

/// Returns the volume of blood of a body of `mass` kilograms, in cubic metres.
pub fn volume(mass: f32) -> f32 {
    SHARE * mass.max(0.0) / DENSITY
}

/// Returns the pressure in the arteries of a body of hip `height` metres under `gravity`, in
/// pascals: the pressure that perfuses the capillaries plus ρ·g·h to lift blood to the head. An open
/// circulation keeps [`OPEN`] of it.
pub fn pressure(gravity: f32, height: f32, closed: bool) -> f32 {
    let lift = DENSITY * gravity.max(0.0) * LIFT * height.max(0.0);
    let all = BASE + lift;
    if closed { all } else { all * OPEN }
}

/// Returns the speed of a jet that leaves an opening under `drop` pascals, in metres per second
/// (Bernoulli: ½ρv² = Δp).
pub fn speed(drop: f32) -> f32 {
    (2.0 * drop.max(0.0) / DENSITY).sqrt()
}

/// Returns the diameter of the main artery of a body of `mass` kilograms, in metres.
pub fn vessel(mass: f32) -> f32 {
    VESSEL * mass.max(1e-3).powf(VESSEL_EXPONENT)
}

/// Returns the flow in cubic metres per second out of a cut of `width` metres across the main artery
/// of a body of `mass` kilograms at `drop` pascals: a slit of that width through a lumen of
/// [`vessel`] diameter, as a sharp-edged orifice.
pub fn gush(width: f32, mass: f32, drop: f32) -> f32 {
    let d = vessel(mass);
    let opening = PI / 4.0 * d * width.clamp(0.0, d);
    DISCHARGE * opening * speed(drop)
}

/// Returns the flow in cubic metres per second out of a shallow cut `width` metres long and `depth`
/// metres deep, which opens veins and capillaries at [`VENOUS`] pressure: the share of its face that
/// is the mouth of vessels is [`PORES`].
pub fn seep(width: f32, depth: f32) -> f32 {
    DISCHARGE * PORES * width.max(0.0) * depth.max(0.0) * speed(VENOUS)
}

/// Returns the share of its first flow a wound keeps `age` seconds after the cut: it decays by
/// e^(-t/τ), slowly when it reached a `major` vessel.
pub fn clot(age: f32, major: bool) -> f32 {
    let tau = if major { CLOT_MAJOR } else { CLOT };
    (-age.max(0.0) / tau).exp()
}
