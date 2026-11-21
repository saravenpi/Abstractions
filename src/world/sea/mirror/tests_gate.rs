//! Tests of when the water mirror draws: off with the setting off, with no water, with the camera
//! under the water or high above it, looking up, or with no actor near.

use bevy::prelude::*;

use super::gate::{Scene, nearest};
use super::tune::{CEILING, REACH};

/// The robot at a lake shore with the camera behind it, looking across the water.
fn shore() -> Scene {
    Scene {
        want: true,
        water: true,
        height: 1.8,
        looking: true,
        nearest: 6.0,
    }
}

/// One condition of the pass taken away from the shore scene.
type Change = fn(&mut Scene);

/// At the shore with the setting on the mirror draws; each missing condition stops it.
#[test]
fn the_pass_runs_only_when_it_shows_something() {
    assert!(shore().pass());
    let cases: [(&str, Change); 8] = [
        ("setting off", |s| s.want = false),
        ("no water", |s| s.water = false),
        ("looking up", |s| s.looking = false),
        ("under water", |s| s.height = -0.5),
        ("at the surface", |s| s.height = 0.0),
        ("high above", |s| s.height = CEILING + 1.0),
        ("no actor near", |s| s.nearest = REACH + 1.0),
        ("no actor", |s| s.nearest = f32::INFINITY),
    ];
    for (why, change) in cases {
        let mut scene = shore();
        change(&mut scene);
        assert!(!scene.pass(), "{why}");
    }
}

/// The nearest actor is measured from the mirror camera, so an actor far above the water is far
/// from it, and with no actor nothing is near.
#[test]
fn the_nearest_actor_is_seen_from_under_the_water() {
    let eye = Vec3::new(0.0, -2.0, 0.0);
    let actors = [Vec3::new(0.0, 1.0, -4.0), Vec3::new(0.0, 400.0, 0.0)];
    assert_eq!(nearest(eye, actors.into_iter()), 5.0);
    assert!(nearest(eye, [Vec3::new(0.0, 400.0, 0.0)].into_iter()) > REACH);
    assert_eq!(nearest(eye, std::iter::empty()), f32::INFINITY);
}
