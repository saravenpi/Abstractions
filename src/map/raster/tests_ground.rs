//! Tests of the flat map as a new player opens it: at the zoom the map opens at on the ground,
//! the land around the home base is lit from edge to edge of what was explored, not as one
//! pixel under the marker of the player.

use bevy::math::DVec3;

use super::tests_tile::tile_at;
use super::tile::{SIDE, TileId, level_for};
use super::tile_paint::lit;
use crate::map::explored::{Explored, REACH_CAP};
use crate::map::level::{GROUND, first};
use crate::universe::{BodyId, EARTH_RADIUS};

/// Returns the share of the pixels of tile `id` within `metres` of `dir` that `explored` lights.
fn share(explored: &Explored, (id, dir): (TileId, DVec3), metres: f64) -> f64 {
    let leaves = explored.leaves(BodyId::default()).expect("ground");
    let lit = lit(leaves, id, EARTH_RADIUS);
    let near = |(px, py): &(usize, usize)| {
        let at = id.direction((*px as f64 + 0.5, *py as f64 + 0.5));
        at.angle_between(dir) * EARTH_RADIUS <= metres
    };
    let all = (0..SIDE).flat_map(|py| (0..SIDE).map(move |px| (px, py)));
    let inside: Vec<(usize, usize)> = all.filter(near).collect();
    let shown = inside.iter().filter(|at| lit.contains(at)).count();
    shown as f64 / inside.len().max(1) as f64
}

/// The map opens on the player at the ground zoom, and the tile under them at that zoom shows
/// the ground explored around the base as a filled disc: every pixel within 1.5 km is lit.
#[test]
fn the_map_opens_on_the_ground_explored_around_the_base() {
    let dir = DVec3::new(0.5, 0.4, -0.76).normalize();
    let mut explored = Explored::default();
    let body = BodyId::default();
    explored.reveal(body, dir, REACH_CAP / EARTH_RADIUS, EARTH_RADIUS);
    let lens = first(None, Some(dir * EARTH_RADIUS));
    assert_eq!(lens.zoom, GROUND);
    let id = tile_at(dir, level_for(lens.zoom));
    assert!(id.pixel(dir).is_some());
    let filled = share(&explored, (id, dir), 1500.0);
    assert!(
        filled > 0.999,
        "{filled} of the pixels within 1.5 km are lit"
    );
}

/// At that zoom the map spans a few kilometres, so the explored ground fills it, and the
/// ground beyond what was explored stays dark.
#[test]
fn the_ground_map_spans_what_a_player_explores() {
    let across = std::f64::consts::TAU * EARTH_RADIUS / f64::from(GROUND);
    assert!(
        (2.0 * REACH_CAP..4.0 * REACH_CAP).contains(&across),
        "{across} m"
    );
    let dir = DVec3::new(0.5, 0.4, -0.76).normalize();
    let mut explored = Explored::default();
    explored.reveal(BodyId::default(), dir, 300.0 / EARTH_RADIUS, EARTH_RADIUS);
    let id = tile_at(dir, level_for(GROUND));
    let all = share(&explored, (id, dir), 1.0e9);
    assert!(
        all > 0.0 && all < 0.5,
        "{all} of the tile is lit by a 300 m walk"
    );
}
