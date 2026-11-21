//! Checks that props merged through a rig keep their true shape wherever the chart of a cube face is squeezed.

use bevy::prelude::*;

use super::*;
use crate::world::scatter::tile::extent;
use crate::world::scatter::zones::trio;
use crate::world::scatter::{Placement, Rig};
use crate::world::solids::tests::triangles;

/// Returns a prop near the rim of its chunk, turned and scaled.
fn sample(plant: Plant) -> Placement {
    Placement {
        pos: Vec3::new(25.0, 12.0, -28.0),
        yaw: 1.1,
        scale: 1.2,
        plant,
        variant: 2,
    }
}

/// A tree, a willow and a boulder merged through a rig start, end and reach as far as the flat ones within 1 mm, at
/// a face centre, at an edge midpoint and next to a corner, and they stand along the true up of the planet.
#[test]
fn props_keep_their_shape_on_the_globe() {
    for tile in trio() {
        let (embed, turn) = (|p| tile.point(p), |p| tile.turn(p));
        let rig = Rig {
            embed: &embed,
            turn: &turn,
        };
        for p in [Plant::Oak, Plant::Willow, Plant::Boulder].map(sample) {
            let flat = extent(&merge_props(&[p]), p.pos, Vec3::Y);
            let (origin, up) = (tile.point(p.pos), tile.turn(p.pos) * Vec3::Y);
            let rigid = extent(&merge_props_on(&[p], &rig), origin, up);
            for k in 0..3 {
                let off = (flat[k] - rigid[k]).abs();
                assert!(
                    off < 1e-3,
                    "{:?} differs by {off} m at {}",
                    p.plant,
                    tile.coord
                );
            }
            let true_up = tile.planet(p.pos).normalize();
            let lean = (tile.frame.rot * up.as_dvec3()).angle_between(true_up);
            assert!(lean < 1e-4, "{:?} leans by {lean} rad", p.plant);
        }
    }
}

/// The point by point warp the rig replaces squeezes the same tree by more than 15 cm across the edge at an
/// edge midpoint, so the check above would catch it.
#[test]
fn a_point_warp_would_squeeze_the_same_tree() {
    let [_, edge, _] = trio();
    let p = sample(Plant::Oak);
    let flat = triangles(&merge_props(&[p]));
    let (origin, south) = (edge.point(p.pos), edge.turn(p.pos) * Vec3::Z);
    let reach = |along: &dyn Fn(Vec3) -> f32| {
        let points = flat.iter().flatten();
        points.map(|v| along(*v).abs()).fold(0.0, f32::max)
    };
    let depth = reach(&|v| (v - p.pos).z);
    let warped = reach(&|v| (edge.point(v) - origin).dot(south));
    assert!(depth - warped > 0.15, "{depth} m against {warped} m");
}
