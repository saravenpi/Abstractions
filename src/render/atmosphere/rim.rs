//! The rim of air around a body seen from space: the halo shell is as thick as the air of the
//! body, and never thinner than a pixel and a half, so it stays a steady line from afar.

use crate::universe::PlanetSpec;

/// Radius of the thickest halo shell in planet radii: the halo of a body whose air is deep against
/// its size, or of a small one far away.
pub const HALO_SCALE: f32 = 1.08;

/// Least thickness of the halo as an angle in radians: a game pixel and a half at the default field
/// of view, 75 degrees over 360 rows.
pub const RIM: f32 = 0.0064;

/// Returns the radius of the halo shell of `planet` in its radii, when its radius covers `angle`
/// radians: the top of its air, at least [`RIM`] above its ground, at most [`HALO_SCALE`].
pub fn halo_scale(planet: &PlanetSpec, angle: f32) -> f32 {
    let air = (planet.atmosphere_height / planet.radius.max(1.0)) as f32;
    let least = RIM / angle.max(f32::MIN_POSITIVE);
    1.0 + air.max(least).min(HALO_SCALE - 1.0)
}

/// Returns the facing of a halo shell of `scale` planet radii where the limb of its planet shows:
/// the sine of the angle the shell leans away from the line of sight there.
pub fn limb(scale: f32) -> f32 {
    (1.0 - 1.0 / (scale * scale)).max(0.0).sqrt()
}
