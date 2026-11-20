//! The reveal rules: what a visit lets the player see.

use bevy::math::DVec3;

use crate::universe::{GALAXY_CELL, GalaxyId, GalaxySpec, StarId, StarSpec, Universe};

/// A visited system reveals every system within this many light-years.
///
/// At the real density near the Sun, 0.004 stars per cubic light-year, a sphere of this radius
/// holds about 17 stars, close to the real census of the 10 light-years around the Sun. A
/// reveal reads about 27 sectors of 10 light-years, each with at most 12 stars.
pub const REVEAL_LY: f64 = 10.0;

/// A visited galaxy reveals every galaxy within this many grid cells on each axis.
pub const REVEAL_CELLS: i32 = 2;

/// Number of galaxies nearest to the home galaxy that a new game knows.
pub const HOME_NEIGHBOURS: usize = 3;

/// Widest search for the nearest galaxies, in grid cells.
const SEARCH_CELLS: i32 = 4;

/// Returns the systems a visit to star `id` reveals, itself included, nearest first.
///
/// One call to `stars_near`: its cost is the sectors of the sphere, never a galaxy.
pub fn systems_around(universe: &Universe, id: StarId) -> Vec<StarSpec> {
    let Some(star) = universe.star(id) else {
        return Vec::new();
    };
    universe.stars_near(id.galaxy, star.pos, REVEAL_LY)
}

/// Returns the galaxies a visit to galaxy `id` reveals, itself included.
pub fn galaxies_around(universe: &Universe, id: GalaxyId) -> Vec<GalaxySpec> {
    universe.galaxies_near(id, REVEAL_CELLS)
}

/// Returns the distance in metres between the centres of two galaxies.
pub fn galaxy_gap(a: &GalaxySpec, b: &GalaxySpec) -> f64 {
    a.locate(DVec3::ZERO).distance(b.locate(DVec3::ZERO))
}

/// Returns the home galaxy followed by the galaxies nearest to it, nearest first.
///
/// The search widens one cell at a time. It stops once the last galaxy kept is nearer than
/// anything outside the cells searched could be.
pub fn home_galaxies(universe: &Universe) -> Vec<GalaxySpec> {
    let id = universe.home().star.galaxy;
    let Some(home) = universe.galaxy(id) else {
        return Vec::new();
    };
    let mut near = Vec::new();
    for cells in 1..=SEARCH_CELLS {
        near = universe.galaxies_near(id, cells);
        near.retain(|galaxy| galaxy.id != id);
        near.sort_by(|a, b| galaxy_gap(a, &home).total_cmp(&galaxy_gap(b, &home)));
        near.truncate(HOME_NEIGHBOURS);
        let safe = f64::from(cells) * GALAXY_CELL;
        if near.len() == HOME_NEIGHBOURS
            && near.iter().all(|galaxy| galaxy_gap(galaxy, &home) <= safe)
        {
            break;
        }
    }
    near.insert(0, home);
    near
}
