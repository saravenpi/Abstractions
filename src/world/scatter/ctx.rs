//! What one pass over a chunk needs to look up the world, read once per chunk.

use bevy::prelude::*;

use super::air::{Air, still};
use super::chunk_center;
use super::cover::Cover;
use super::density::Fields;
use super::grow::Flora;
use super::metric::Metric;
use super::relief::Relief;
use crate::world::Terrain;

/// What one pass over a chunk needs to look up the world.
pub struct Ctx<'a> {
    /// The view of the planet the chunk belongs to.
    pub terrain: &'a Terrain,
    /// Chart position of the chunk center.
    pub center: Vec2,
    /// Patch noise behind the plant densities.
    pub fields: Fields,
    /// True-metre measure of the chunk.
    pub metric: Metric,
    /// Visible ground of the chunk. The legacy plane has none and reads the analytic ground.
    pub relief: Option<Relief<'a>>,
    /// Climate over the chunk. The legacy plane has none and grows as it always did.
    pub air: Option<Air>,
    /// The plant species of a living or dead planet, or none where the tables of home grow.
    pub flora: Option<Flora>,
    /// The plant cover over the chunk of a living planet, or none where the tables of home grow.
    pub cover: Option<Cover>,
    /// Chart position and room of every grown big plant already placed, whose shade the undergrowth seeks.
    pub canopy: Vec<(Vec2, f32)>,
}

impl<'a> Ctx<'a> {
    /// Returns the context of chunk `coord` of `terrain` over its visible ground `relief`, if any: its noise,
    /// measure, climate, flora and plant cover, with no canopy yet.
    pub fn new(terrain: &'a Terrain, coord: IVec2, relief: Option<Relief<'a>>) -> Self {
        let center = chunk_center(coord);
        let air = relief
            .as_ref()
            .filter(|_| !still())
            .map(|_| Air::new(terrain, coord));
        let flora = Flora::of(terrain, center);
        let patchy = flora
            .as_ref()
            .filter(|f| !f.growers.is_empty())
            .map(|f| f.patchiness);
        Self {
            terrain,
            center,
            fields: Fields::new(terrain.seed()),
            metric: Metric::of(terrain, center),
            cover: patchy.map(|p| Cover::new(terrain, coord, p, air.as_ref())),
            air,
            flora,
            relief,
            canopy: Vec::new(),
        }
    }
}
