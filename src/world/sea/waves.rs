//! Wave parameters derived from the wind: four directional sine waves around the wind direction.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::wind::Wind;

/// Number of waves summed by the water shader.
pub const WAVE_COUNT: usize = 4;

/// Share of the total amplitude carried by each wave, longest wave first.
const SHARES: [f32; WAVE_COUNT] = [0.45, 0.30, 0.15, 0.10];

/// Wavelength in meters of each wave in calm wind.
const WAVELENGTHS: [f32; WAVE_COUNT] = [22.0, 11.0, 6.0, 4.0];

/// Angle in radians of each wave away from the wind direction.
const OFFSETS: [f32; WAVE_COUNT] = [0.0, 0.35, -0.5, 0.9];

/// Sum of all amplitudes in meters when the wind is still.
const CALM_HEIGHT: f32 = 0.05;

/// Extra summed amplitude in meters at full wind strength.
const GUST_HEIGHT: f32 = 0.13;

/// One directional wave.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wave {
    /// Unit direction the wave travels toward, on the XZ plane.
    pub direction: Vec2,
    /// Distance between two crests in meters.
    pub wavelength: f32,
    /// Crest height above the mean level in meters.
    pub amplitude: f32,
}

impl Wave {
    /// Packs the wave as direction x, direction z, wave number and amplitude for the shader.
    pub fn packed(&self) -> Vec4 {
        Vec4::new(
            self.direction.x,
            self.direction.y,
            TAU / self.wavelength,
            self.amplitude,
        )
    }
}

/// Returns the waves the wind raises: gentle and short in calm air, taller and longer in gusts.
pub fn waves_for(wind: &Wind) -> [Wave; WAVE_COUNT] {
    let strength = wind.strength.clamp(0.0, 1.0);
    let total = CALM_HEIGHT + GUST_HEIGHT * strength;
    let stretch = 0.8 + 0.2 * strength;
    std::array::from_fn(|i| Wave {
        direction: Vec2::from_angle(OFFSETS[i]).rotate(wind.direction),
        wavelength: WAVELENGTHS[i] * stretch,
        amplitude: total * SHARES[i],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Heights grow with the wind and stay gentle, wavelengths stay in range, directions stay unit.
    #[test]
    fn waves_follow_the_wind() {
        let calm = waves_for(&Wind {
            direction: Vec2::X,
            strength: 0.0,
        });
        let gust = waves_for(&Wind {
            direction: Vec2::new(0.6, 0.8),
            strength: 1.0,
        });
        let sum = |w: &[Wave; WAVE_COUNT]| w.iter().map(|w| w.amplitude).sum::<f32>();
        assert!((sum(&calm) - 0.05).abs() < 1e-6 && (sum(&gust) - 0.18).abs() < 1e-6);
        for wave in calm.iter().chain(gust.iter()) {
            assert!((3.0..=25.0).contains(&wave.wavelength));
            assert!((wave.direction.length() - 1.0).abs() < 1e-5);
        }
        assert!((calm[0].direction - Vec2::X).length() < 1e-6);
    }
}
