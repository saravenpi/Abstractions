//! How well life does at a place: the climate field read where plants and settlers stand, and what its
//! warmth and wetness mean for each biome.

use bevy::math::{DVec3, Vec2};

use crate::climate::{self, Climate, Setting, Spot};
use crate::universe::HOME_DAY_SECONDS;
use crate::world::Terrain;
use crate::world::biome::Biome;
use crate::world::globe::{Chart, Globe, Site};

/// Season every placement reads: the middle of spring, when neither hemisphere leans to the star.
pub const SEASON: f32 = 0.25;

/// Value of the continent measure that counts as fully inland for the climate.
const INLAND: f32 = 0.45;

/// Coldest and warmest air in kelvin a village is built in.
pub const VILLAGE_AIR: (f32, f32) = (265.0, 315.0);

/// Air in kelvin over which a desert with less moisture than the given wetness is a sea of dunes,
/// its heart, where nobody camps.
pub const DUNE_SEA: (f32, f32) = (295.0, 0.15);

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

/// Returns the climate setting of `globe`. The globe keeps no day length and no orbit, so the day of
/// home stands in and no moon is read as tidally locked.
fn setting(globe: &Globe) -> Setting {
    let record = globe.record();
    Setting {
        temperature: record.temperature as f32,
        pressure: record.pressure as f32,
        ocean: record.ocean as f32,
        tilt: record.tilt as f32,
        day_seconds: HOME_DAY_SECONDS as f32,
        gravity: globe.gravity() as f32,
        radius: globe.radius() as f32,
        seed: globe.seed(),
        near: None,
    }
}

/// Returns the planet-frame east and north at unit direction `dir`, east being where the spin carries
/// the ground, and plain axes at a pole.
fn compass(dir: DVec3) -> (DVec3, DVec3) {
    let east = DVec3::Y.cross(dir).try_normalize().unwrap_or(DVec3::X);
    (east, dir.cross(east))
}

/// Returns the climate of `globe` in planet-frame unit direction `dir`, on ground `altitude` metres
/// above the liquid surface (on a dry world, above the datum), whose rise per metre is `rise` in
/// planet axes, at [`SEASON`].
pub fn climate_on(globe: &Globe, dir: DVec3, altitude: f32, rise: DVec3) -> Climate {
    let (east, north) = compass(dir);
    let spot = Spot {
        dir,
        altitude: altitude.max(globe.sea_level()),
        inland: globe.shape.base((dir * globe.radius()).point()) / INLAND,
        slope: Vec2::new(rise.dot(east) as f32, rise.dot(north) as f32),
    };
    climate::at(&setting(globe), &spot, SEASON)
}

/// Returns the climate at chart position `w` of `terrain`, read as on level ground, or nothing on the
/// legacy plane, which has no latitude.
pub fn climate_at(terrain: &Terrain, w: Vec2) -> Option<Climate> {
    let chart = terrain.chart();
    let dir = chart.dir(w.x, w.y);
    (chart != Chart::Flat)
        .then(|| climate_on(terrain.globe(), dir, terrain.height(w.x, w.y), DVec3::ZERO))
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

/// Returns true when `air` is neither too cold nor too hot for a village.
pub fn livable(air: &Climate) -> bool {
    (VILLAGE_AIR.0..=VILLAGE_AIR.1).contains(&air.temperature)
}

/// Returns true when chart position `w` of `terrain` lies in the heart of a desert, a hot sea of
/// dunes. The legacy plane has no climate and no such heart.
pub fn dune_sea(terrain: &Terrain, w: Vec2) -> bool {
    let hot = |air: Climate| air.temperature > DUNE_SEA.0 && air.moisture < DUNE_SEA.1;
    terrain.biome(w.x, w.y) == Biome::Desert && climate_at(terrain, w).is_some_and(hot)
}
