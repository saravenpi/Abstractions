//! Tests that the base is solid where it is drawn: its roofs carry a body, and a body that glides into
//! them from the side is stopped.

use bevy::math::Affine3A;
use bevy::prelude::*;

use super::plan::draw;
use super::spawn::solid;
use super::tests::base;
use super::{container, station};
use crate::world::structures::touch::{BALL, dropped, pushed, stage};

/// Height in metres of the top of the canopy of the station above its origin.
const CANOPY: f32 = 2.63;

/// Returns a physics stage that holds the solid boxes of the base of the world of `seed`, in the frame
/// of the base, with the poses of its container and of its station.
fn raised(seed: u64) -> (App, Affine3A, Affine3A) {
    let (base, globe) = base(seed);
    let lot = base.lot(&globe);
    let app = stage(solid(&draw(&lot).blocks));
    (app, container::pose(&lot), station::pose(&lot))
}

/// A body dropped on the roof of the container comes to rest on it.
#[test]
fn bodies_rest_on_the_container_roof() {
    for seed in [1, 7, 42] {
        let (mut app, container, _) = raised(seed);
        let roof = container.transform_point3(Vec3::new(0.6, container::SIZE.y, 0.2));
        let (at, speed) = dropped(&mut app, roof + Vec3::Y * 2.0);
        let off = at - roof - Vec3::Y * BALL;
        assert!(
            off.length() < 0.05 && speed < 0.05,
            "seed {seed}: the ball is {off} from the roof at {speed} m/s"
        );
    }
}

/// A body dropped on the canopy of the station comes to rest on it.
#[test]
fn bodies_rest_on_the_station_canopy() {
    for seed in [1, 7, 42] {
        let (mut app, _, station) = raised(seed);
        let roof = station.transform_point3(Vec3::new(0.5, CANOPY, 0.3));
        let (at, speed) = dropped(&mut app, roof + Vec3::Y * 2.0);
        let off = at - roof - Vec3::Y * BALL;
        assert!(
            off.length() < 0.05 && speed < 0.05,
            "seed {seed}: the ball is {off} from the canopy at {speed} m/s"
        );
    }
}

/// A body that glides level into the edge of a roof of the base, at wingsuit speed, is stopped by it: into
/// the canopy of the station between its posts, and into the top of the end wall of the container.
#[test]
fn glides_into_the_roofs_are_stopped() {
    for seed in [1, 7, 42] {
        let (mut app, container, station) = raised(seed);
        let cases = [
            ("canopy", station, CANOPY - 0.08, 2.2),
            ("container", container, container::SIZE.y - 0.4, 3.0),
        ];
        for (what, pose, height, edge) in cases {
            let from = pose.transform_point3(Vec3::new(-edge - 6.0, height, 0.0));
            let along = pose.transform_vector3(Vec3::X);
            let end = pushed(&mut app, from, along * 22.0, (60, 60.0));
            let reached = pose.inverse().transform_point3(end).x;
            assert!(
                reached < -edge,
                "seed {seed}: the glide into the {what} reached x = {reached}"
            );
        }
    }
}
