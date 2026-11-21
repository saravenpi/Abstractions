//! Tests of a carcass lying on the ground: drawn again as it streams in, or fallen where it was killed,
//! facing any way, on flat ground and on a slope, every part of it lies on the ground or on the body,
//! none hanging in the air.

use bevy::math::{DVec3, Quat, Vec2, Vec3};

use super::fixtures::quadruped_with;
use super::pose::Placed as Pose;
use super::rig::Motion;
use super::tests_size::HIDE;
use crate::fauna::attack::Lain;
use crate::fauna::beast::lay;
use crate::fauna::place::Spot;
use crate::life::body::{Body, soup};
use crate::space::Placed;

/// Headings the carcasses are tried at: the eight points of the compass.
const WAYS: usize = 8;

/// Returns the grounds the carcasses are tried on: flat, and a slope rising 0.3 m per metre east and
/// 0.1 m per metre north.
fn grounds() -> [fn(Vec2) -> f32; 2] {
    [|_| 0.0, |p| 0.3 * p.x + 0.1 * p.y]
}

/// Returns the lowest and the highest point over `ground` of the drawn parts of `body`, posed `pose`
/// under a root laid at `placed` in `spot`.
fn span(
    spot: &Spot,
    (body, pose): (&Body, &[Pose]),
    placed: Placed,
    ground: fn(Vec2) -> f32,
) -> (f32, f32) {
    let origin = spot.local(placed.pos);
    let turn = (spot.rot.inverse() * placed.rot).as_quat();
    let (mut low, mut high) = (f32::MAX, f32::MIN);
    for (part, (at, rot, scale)) in body.parts.iter().zip(pose) {
        for corner in soup(part, &HIDE).faces.iter().flat_map(|f| f.corners) {
            let p = origin + turn * (*at + *rot * (corner * *scale));
            let over = p.y - ground(Vec2::new(p.x, p.z));
            (low, high) = (low.min(over), high.max(over));
        }
    }
    (low, high)
}

/// Asserts that a carcass of `body` spans `low` to `high` over the ground: nothing more than 8 cm in
/// it, nothing higher than its hips, as a body lying on its side is.
fn lies(body: &Body, (low, high): (f32, f32), what: &str) {
    assert!(low > -0.08, "{what}: a part {low} m in the ground");
    assert!(
        high < body.height,
        "{what}: a part {high} m in the air, the hips are {} m",
        body.height
    );
}

/// A carcass drawn again as it streams in lies on its side along the way it faces, at every heading, on
/// flat ground and on a slope.
#[test]
fn a_carcass_drawn_again_lies_on_the_ground() {
    let body = quadruped_with(1, 0.4);
    let spot = Spot::at(DVec3::Y, 6.0e6);
    for (g, ground) in grounds().into_iter().enumerate() {
        for k in 0..WAYS {
            let facing =
                Quat::from_rotation_y(k as f32 * std::f32::consts::TAU / WAYS as f32) * Vec3::NEG_Z;
            let at = Vec3::new(2.0, ground(Vec2::new(2.0, -1.0)), -1.0);
            let mut pose = Motion::new(&body, at, facing, 3).pose(&body);
            let (_, lain) = lay(&spot, &body, (at, facing), &mut pose);
            let placed = lain.expect("a carcass lies").rest(&spot, &ground);
            lies(
                &body,
                span(&spot, (&body, &pose), placed, ground),
                &format!("ground {g}, heading {k}"),
            );
        }
    }
}

/// An animal killed where it stood falls on its side about the way it faced and lies there, at every
/// heading, on flat ground and on a slope.
#[test]
fn a_killed_animal_lies_on_the_ground() {
    let body = quadruped_with(1, 0.4);
    let spot = Spot::at(DVec3::Y, 6.0e6);
    for (g, ground) in grounds().into_iter().enumerate() {
        for k in 0..WAYS {
            let facing =
                Quat::from_rotation_y(k as f32 * std::f32::consts::TAU / WAYS as f32) * Vec3::NEG_Z;
            let at = Vec3::new(-1.0, ground(Vec2::new(-1.0, 3.0)), 3.0);
            let pose = Motion::new(&body, at, facing, 5).pose(&body);
            let side = (0.3 * body.height).max(0.05);
            let tipped = Quat::from_axis_angle(facing, std::f32::consts::FRAC_PI_2);
            let lain = Lain::tipped(at, tipped, (side, body.height));
            let placed = lain.rest(&spot, &ground);
            lies(
                &body,
                span(&spot, (&body, &pose), placed, ground),
                &format!("ground {g}, heading {k}"),
            );
        }
    }
}
