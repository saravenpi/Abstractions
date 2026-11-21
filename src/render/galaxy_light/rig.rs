//! What the tests of the galaxy map share: the home galaxy and the reading of a baked map.

use bevy::math::DVec3;

use super::march::{SIDE, bake};
use super::pack::decode;
use super::plan;
use crate::universe::{GalaxySpec, Universe};
use crate::weather::face_dir;

/// Returns the universe of seed 7 and its home galaxy, a spiral.
pub(super) fn home() -> (Universe, GalaxySpec) {
    let universe = Universe::new(7);
    let galaxy = universe.home().star.galaxy;
    let spec = universe.galaxy(galaxy).expect("the home galaxy");
    (universe, spec)
}

/// Returns the light of texel `column`, `row` of face `face` of a map `side` texels wide: the mean
/// of its three channels.
pub(super) fn light(bytes: &[u8], side: usize, (face, column, row): (usize, usize, usize)) -> f32 {
    let first = ((face * side + row) * side + column) * 4;
    bytes[first..first + 3]
        .iter()
        .map(|b| decode(*b))
        .sum::<f32>()
        / 3.0
}

/// Returns the unit direction of texel `column`, `row` of face `face` of a map `side` texels
/// wide, and the share of the sky it covers against the texel at the middle of a face.
pub(super) fn texel(side: usize, (face, column, row): (usize, usize, usize)) -> (DVec3, f32) {
    let at = |index: usize| 2.0 * (index as f64 + 0.5) / side as f64 - 1.0;
    let (x, y) = (at(column), at(row));
    (
        face_dir(face, x, y),
        (1.0 + x * x + y * y).powf(-1.5) as f32,
    )
}

/// Returns every texel of a map with its direction, its weight on the sky and its light.
pub(super) fn texels(bytes: &[u8], side: usize) -> Vec<(DVec3, f32, f32)> {
    let all = (0..6).flat_map(|f| (0..side).flat_map(move |r| (0..side).map(move |c| (f, c, r))));
    all.map(|at| {
        let (dir, weight) = texel(side, at);
        (dir, weight, light(bytes, side, at))
    })
    .collect()
}

/// Returns the texels of the map of the home galaxy alone seen from a place 0.4 radii out along
/// the x axis of it, in its disc, where the real core lies along the negative x axis. The
/// galaxies far outside are left out: a core far away is as bright on the sky, a point at a time,
/// as the core of the galaxy the camera is in.
pub(super) fn halfway() -> Vec<(DVec3, f32, f32)> {
    let (universe, spec) = home();
    let (mut sources, _) = plan(&universe, spec.id, DVec3::X * 0.4 * spec.radius);
    sources.retain(|source| source.inside);
    texels(&bake(&sources), SIDE)
}
