//! The ground of the test planet: hills, bays and seas, a temperate meadow climate and a 1200 s day.

use bevy::math::DVec3;

use crate::life::census::Site;
use crate::world::Biome;

/// Returns a site on land or sea at `height` metres, in a temperate meadow under `daylight`.
pub fn site(height: f32, daylight: f32) -> Site {
    Site {
        height,
        temperature: 288.0,
        moisture: 0.5,
        biome: Biome::Meadow,
        liquid: height < 0.0,
        daylight,
    }
}

/// Returns the ground height in metres of a test planet of hills, bays and seas at `dir`.
pub fn height(dir: DVec3) -> f32 {
    let d = dir.as_vec3();
    300.0 * ((3.0 * d.x).sin() * (2.0 * d.y).cos() + 0.3 * (5.0 * d.z).sin()) - 40.0
}

/// Returns the site of the test planet at `dir` under `daylight`.
pub fn sample(dir: DVec3, daylight: f32) -> Site {
    site(height(dir), daylight)
}

/// Returns the daylight of the test planet at `dir` and universe `time`: a day of 1200 s whose noon
/// moves west with longitude.
pub fn light(dir: DVec3, time: f64) -> f32 {
    let longitude = dir.z.atan2(dir.x);
    (0.5 + 0.6 * (time / 1200.0 * std::f64::consts::TAU + longitude).sin()).clamp(0.0, 1.0) as f32
}
