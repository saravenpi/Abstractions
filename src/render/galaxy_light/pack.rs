//! The bytes of the galaxy map: a light of any brightness in the four bytes of a texel.

use bevy::math::Vec3;

/// Returns a light as the bytes of a texel: each channel as `1 - exp(-light)`, so that any
/// brightness fits, and the whole texel opaque.
pub fn encode(light: Vec3) -> [u8; 4] {
    let byte = |value: f32| (255.0 * (1.0 - (-value.max(0.0)).exp())).round() as u8;
    [byte(light.x), byte(light.y), byte(light.z), 255]
}

/// Returns the light a byte of [`encode`] stands for, as the sky shader reads it.
#[cfg(test)]
pub fn decode(byte: u8) -> f32 {
    -(1.0 - f32::from(byte) / 255.0).max(1.0e-3).ln()
}
