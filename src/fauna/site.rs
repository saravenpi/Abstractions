//! The planet as herds read it: the site at a unit direction from the globe and its climate, and the
//! daylight at a unit direction and a universe time from the orbit and the spin of the planet.
//!
//! Both are pure functions of the planet and universe time, so every player rolls the same herds.

use bevy::math::DVec3;

use crate::climate::{self, MEAN_SEASON, Spot};
use crate::life::census::Site;
use crate::render::smoothstep;
use crate::universe::{PlanetSpec, sun};
use crate::world::{Globe, Surface};

/// Heights of the star, as the sine of its angle above the horizon, between which the day comes up:
/// the edges of `crate::render::daylight`.
const DAWN: (f32, f32) = (-0.12, 0.25);

/// Returns the daylight, 0 at night to 1 by day, at unit direction `dir` on `planet` at universe
/// `time`.
pub fn light(planet: &PlanetSpec, dir: DVec3, time: f64) -> f32 {
    let height = dir.dot(sun(planet, time)) as f32;
    smoothstep(DAWN.0, DAWN.1, height)
}

/// Returns true when `surface` is liquid to an animal: the sea, a pond or a river.
fn wet(surface: Surface) -> bool {
    matches!(
        surface,
        Surface::Water | Surface::Shallows | Surface::Reef | Surface::Pond | Surface::River
    )
}

/// Returns the site at unit direction `dir` of `globe`, the planet `planet`, at universe `time`.
pub fn site(globe: &Globe, planet: &PlanetSpec, dir: DVec3, time: f64) -> Site {
    let height = globe.height(dir);
    let ground = globe.spot(dir);
    let spot = Spot {
        altitude: height.max(ground.altitude),
        ..ground
    };
    let climate = climate::at(globe.setting(), &spot, MEAN_SEASON);
    let sea = height < globe.sea_level();
    Site {
        height,
        temperature: climate.temperature,
        moisture: climate.moisture.clamp(0.0, 1.0),
        biome: globe.biome(dir),
        liquid: sea || wet(globe.surface(dir)),
        daylight: light(planet, dir, time),
    }
}
