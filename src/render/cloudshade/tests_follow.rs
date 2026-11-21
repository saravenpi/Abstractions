//! Tests that the cloud shadows follow the drawn clouds: the shadow at a ground point is the
//! cloud along the sun ray at the same second, and it moves with that cloud and never jumps.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::rig::{RADIUS, cast, ground};
use super::sunward_hit;
use super::tune::{OPTICAL_DEPTH, SUN_EDGE};
use crate::weather::{CELL, density_at};

/// Returns the planet-fixed direction where the ray from `ground` toward `sun` crosses the shell
/// of `shell` metres, found by halving in 64 bits, apart from [`sunward_hit`].
fn crossing(ground: DVec3, sun: DVec3, shell: f64) -> DVec3 {
    let (mut low, mut high) = (0.0, 4.0 * shell);
    for _ in 0..200 {
        let middle = 0.5 * (low + high);
        match (ground + sun * middle).length() < shell {
            true => low = middle,
            false => high = middle,
        }
    }
    (ground + sun * low).normalize()
}

/// The shadow at a ground point is what the drawn cloud holds along the sun ray to that point,
/// at the same second: the beam through the density of the field where the ray meets the shell.
#[test]
fn the_shadow_is_the_drawn_cloud_along_the_sun() {
    for (height, cover, seconds) in [(0.8, 0.6, 50.0), (0.4, 0.9, 7200.0), (0.95, 0.4, 3.6e6)] {
        let c = cast(height, cover, seconds);
        for dir in ground().step_by(7) {
            let at = (dir * RADIUS).as_dvec3();
            let hit = crossing(at, c.sun.as_dvec3(), f64::from(c.shell));
            let cloud = density_at(hit, c.shell / CELL, seconds, cover);
            let lift = crate::render::smoothstep(SUN_EDGE.0, SUN_EDGE.1, dir.dot(c.sun));
            let want = 1.0 - lift * (1.0 - (-OPTICAL_DEPTH * cloud).exp());
            assert!((c.through(dir) - want).abs() < 0.02, "{dir} at {seconds}");
        }
    }
    assert_eq!(
        sunward_hit(Vec3::Y * (RADIUS + 4000.0), Vec3::Y, RADIUS + 3000.0),
        None
    );
}

/// The shadow rides its cloud: a frame later it barely changes, and after a minute it is found
/// again where the wind has carried it, 240 m east, far closer than where it was. Its finer
/// detail slides on a little, so the match is close and not exact.
#[test]
fn the_shadow_moves_with_its_cloud() {
    let (mut frame, mut carried, mut stayed) = (0.0f32, 0.0f32, 0.0f32);
    for dir in ground() {
        let at = |seconds: f64| cast(0.95, 0.6, seconds).through(dir);
        let east = DQuat::from_rotation_y(240.0 / f64::from(RADIUS + 3000.0)) * dir.as_dvec3();
        let there = cast(0.95, 0.6, 1060.0).through(east.as_vec3());
        frame = frame.max((at(1000.0 + 1.0 / 60.0) - at(1000.0)).abs());
        carried += (there - at(1000.0)).abs() / 400.0;
        stayed += (at(1060.0) - at(1000.0)).abs() / 400.0;
    }
    assert!(frame < 0.01, "a frame moves the shadow by {frame}");
    assert!(stayed > 0.03, "the shadow stood still: {stayed}");
    assert!(
        carried < 0.5 * stayed,
        "carried {carried}, left behind {stayed}"
    );
}
