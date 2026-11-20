//! Drops: the size of a drop of blood, the drag it feels in the air and the stain it leaves.

use std::f32::consts::PI;

use super::tune::{AIR, DENSITY, FILM, SPHERE};

/// Returns the radius in metres of a sphere of blood of `volume` cubic metres.
pub fn drop_radius(volume: f32) -> f32 {
    (3.0 * volume.max(0.0) / (4.0 * PI)).cbrt()
}

/// Returns the volume in cubic metres of a sphere of blood of `radius` metres.
pub fn drop_volume(radius: f32) -> f32 {
    4.0 / 3.0 * PI * radius.max(0.0).powi(3)
}

/// Returns the diameter in metres of the stain a drop of `volume` cubic metres leaves on the ground:
/// it spreads to a film of [`FILM`] thickness.
pub fn stain(volume: f32) -> f32 {
    2.0 * (volume.max(0.0) / (FILM * PI)).sqrt()
}

/// Returns the drag a sphere of blood of `radius` metres feels in air `thick` times as dense as at
/// home, as the share of its speed it loses per metre it travels: ½·ρa·Cd·A / m.
pub fn drag(radius: f32, thick: f32) -> f32 {
    let r = radius.max(1e-4);
    0.5 * AIR * thick.max(0.0) * SPHERE * PI * r * r / (DENSITY * 4.0 / 3.0 * PI * r * r * r)
}
