//! How well life does at a place: the climate field read where plants stand, and what its
//! warmth and wetness mean for each biome.

use bevy::math::{DVec3, Vec2};

use crate::climate::{self, Climate};
use crate::world::Terrain;
use crate::world::biome::Biome;
use crate::world::globe::{Chart, Globe, MEAN_SEASON};

/// Temperature in kelvin under which even the pines of the taiga stop: the tree line of the cold.
/// Above it a taiga is a forest of snow pines, colder than the snow line of the other biomes.
pub const TREE_LINE: f32 = 255.0;

/// Share of its plants a biome keeps at the far edge of its warmth and in the driest air.
const FLOOR: f32 = 0.3;

/// The air each biome thrives in: the temperature of its heart in kelvin, how many kelvin away its
/// plants are at their thinnest, and the wetness under which they start to thin out.
#[rustfmt::skip]
const COMFORT: [(f32, f32, f32); 6] = [
    (290.0, 12.0, 0.25),
    (285.0, 14.0, 0.6),
    (278.0, 10.0, 0.2),
    (306.0, 10.0, 0.1),
    (262.0, 8.0, 0.3),
    (304.0, 8.0, 0.8),
];

/// Returns the annual mean climate of `globe` in planet-frame unit direction `dir`, read at the place the
/// globe picks its biomes from, so plants feel the climate their biome was chosen for.
pub fn climate_on(globe: &Globe, dir: DVec3) -> Climate {
    climate::at(globe.setting(), &globe.spot(dir), MEAN_SEASON)
}

/// Returns the annual mean climate at chart position `w` of `terrain`, or nothing on the legacy plane,
/// which has no latitude.
pub fn climate_at(terrain: &Terrain, w: Vec2) -> Option<Climate> {
    let chart = terrain.chart();
    let dir = chart.dir(w.x, w.y);
    (chart != Chart::Flat).then(|| climate_on(terrain.globe(), dir))
}

/// Returns how thick the plants of `biome` grow in `air`, from about 0.1 to 1: full in the heart of
/// its warmth with wet enough air, thinner towards the warmth of the next biome and in dry air.
pub fn vigor(biome: Biome, air: &Climate) -> f32 {
    let (heart, reach, wet) = COMFORT[biome as usize];
    let off = (air.temperature - heart) / reach;
    let warmth = FLOOR + (1.0 - FLOOR) * (1.0 - off * off).max(0.0);
    warmth * (FLOOR + (1.0 - FLOOR) * (air.moisture / wet).clamp(0.0, 1.0))
}

/// Returns true when nothing grows in `biome` where the climate is `air` and only stones lie:
/// under the snow line of the climate, or under the tree line in the taiga, whose pines stand
/// in the snow.
pub fn frozen(biome: Biome, air: &Climate) -> bool {
    match biome {
        Biome::Taiga => air.temperature < TREE_LINE,
        _ => climate::snow(air),
    }
}
