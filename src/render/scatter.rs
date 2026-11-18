//! What the air does to the light of a star: how much of each colour gets through at a given
//! height of the star, and the colour the air scatters back as sky.
//!
//! The lights and the sky colours are derived from these, so a low star reddens and dims by
//! the length of its path through the air, whatever the colour of the star.

use bevy::prelude::*;

/// Optical depth of one air mass for red, green and blue light: blue is scattered most.
pub const DEPTH: Vec3 = Vec3::new(0.045, 0.10, 0.23);

/// Air masses on the path to a star on the horizon. Straight up is one.
pub const HORIZON_MASS: f32 = 12.5;

/// Weights of red, green and blue in the brightness of a light.
pub const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

/// Returns the air masses light crosses from a star whose height above the horizon has the
/// sine `height`. Under the horizon the path is the one of the horizon.
pub fn air_mass(height: f32) -> f32 {
    1.0 / (height.max(0.0) + 1.0 / HORIZON_MASS)
}

/// Returns the share of red, green and blue light that gets through `share` of the air from a
/// star at the sine `height` above the horizon.
pub fn transmit(height: f32, share: f32) -> Vec3 {
    (-DEPTH * (air_mass(height) * share)).exp()
}

/// Returns the colour `light` scaled so that its strongest channel is 1, or white for no light.
pub fn peak(light: Vec3) -> Vec3 {
    let top = light.max_element();
    if top <= 1.0e-6 {
        return Vec3::ONE;
    }
    light / top
}

/// Returns the colour the air scatters out of `light`: the sky straight up, at full strength.
pub fn scattered(light: Vec3) -> Vec3 {
    peak(light * DEPTH)
}

/// Returns the brightness of a light.
pub fn luminance(light: Vec3) -> f32 {
    light.dot(LUMA)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A high star stays nearly white, a low one turns orange and dims, and the sky is blue.
    #[test]
    fn low_light_reddens_and_dims() {
        let (high, low) = (transmit(1.0, 1.0), transmit(0.0, 1.0));
        assert!(high.min_element() > 0.75 && luminance(low) < 0.5 * luminance(high));
        let tint = peak(low);
        assert!(tint.x == 1.0 && tint.y < 0.6 && tint.z < 0.2);
        let sky = scattered(Vec3::ONE);
        assert!(sky.z == 1.0 && sky.x < 0.3 && sky.y < 0.6);
        assert_eq!(peak(Vec3::ZERO), Vec3::ONE);
    }
}
