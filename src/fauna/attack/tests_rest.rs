//! Tests of a carcass at rest: on a slope it lies tilted with the ground, its underside resting on it,
//! and when the ground under it is drawn anew it lies on the new ground.

use bevy::math::{DVec3, Vec2, Vec3};
use bevy::prelude::Quat;

use super::rest::Lain;
use crate::fauna::place::Spot;
use crate::space::Placed;

/// Half sizes in metres of the torso box of the test carcass, in its pose.
const HALF: Vec3 = Vec3::new(0.25, 0.25, 0.7);

/// Metres the middle of the test carcass rests over the ground.
const SIDE: f32 = 0.3;

/// Returns a flat frame on a planet of 6000 km.
fn spot() -> Spot {
    Spot::at(DVec3::Y, 6.0e6)
}

/// Returns a slope rising 0.3 m per metre east and 0.1 m per metre north.
fn slope(p: Vec2) -> f32 {
    0.3 * p.x + 0.1 * p.y
}

/// Returns how a carcass drawn again lies at `(3, -2)`: on its side about the middle of its torso.
fn drawn_again() -> Lain {
    Lain {
        pivot: Vec3::ZERO,
        turn: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        at: Vec2::new(3.0, -2.0),
        middle: Vec2::ZERO,
        side: SIDE,
        reach: 0.9,
    }
}

/// Returns the heights over `ground` of the corners of the torso box laid at `placed` in `spot`, lowest
/// first.
fn clearances(spot: &Spot, placed: Placed, ground: &dyn Fn(Vec2) -> f32) -> Vec<f32> {
    let origin = spot.local(placed.pos);
    let turn = (spot.rot.inverse() * placed.rot).as_quat();
    let signs = [-1.0, 1.0];
    let corners = signs.iter().flat_map(|&x| {
        signs
            .iter()
            .flat_map(move |&y| signs.map(|z| Vec3::new(x, y, z)))
    });
    let mut heights: Vec<f32> = corners
        .map(|c| origin + turn * (c * HALF))
        .map(|p| p.y - ground(Vec2::new(p.x, p.z)))
        .collect();
    heights.sort_by(f32::total_cmp);
    heights
}

/// A carcass on a slope lies with its whole underside on the ground: its four lowest corners stand at
/// one height over the slope, within a centimetre, and that height is over the ground and under its middle.
#[test]
fn a_carcass_lies_tilted_on_a_slope() {
    let spot = spot();
    let placed = drawn_again().rest(&spot, &slope);
    let under = clearances(&spot, placed, &slope);
    println!("corners over the slope: {under:?}");
    assert!(
        under[3] - under[0] < 0.01,
        "the underside is not on the slope: {under:?}"
    );
    assert!(
        (-0.01..SIDE).contains(&under[0]),
        "the underside is {} m over the slope",
        under[0]
    );
}

/// A body that tipped over about a point of its pose rests that point a side over the slope, along the
/// normal of the slope, wherever its pose had it.
#[test]
fn a_fallen_body_rests_its_pivot_on_the_slope() {
    let spot = spot();
    let pivot = Vec3::new(1.0, 2.5, 0.5);
    let lain = Lain::tipped(
        pivot,
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        (SIDE, 1.0),
    );
    let placed = lain.rest(&spot, &slope);
    let origin = spot.local(placed.pos);
    let turn = (spot.rot.inverse() * placed.rot).as_quat();
    let rests = origin + turn * pivot;
    let normal = Vec3::new(-0.3, 1.0, -0.1).normalize();
    let foot = rests - normal * SIDE;
    let gap = foot.y - slope(Vec2::new(foot.x, foot.z));
    assert!(
        gap.abs() < 1e-3,
        "the pivot rests {gap} m off its side over the slope"
    );
}

/// When the ground under a carcass is drawn anew 15 cm higher, it lies 15 cm higher, turned the same.
#[test]
fn a_carcass_follows_the_ground_drawn_anew() {
    let spot = spot();
    let lain = drawn_again();
    let before = lain.rest(&spot, &slope);
    let after = lain.rest(&spot, &|p: Vec2| slope(p) + 0.15);
    let rise = spot.local(after.pos) - spot.local(before.pos);
    assert!((rise - Vec3::Y * 0.15).length() < 1e-3, "moved by {rise}");
    assert!(
        before.rot.angle_between(after.rot) < 1e-6,
        "turned when the ground was redrawn"
    );
}
