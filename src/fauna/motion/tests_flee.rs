//! Tests of a flight: a walker fleeing at 8 m/s runs 200 m over ground drawn at every level, from the
//! 2 m grid of the near chunks to the 16 m grid of sparse coarse nodes, near the robot then far from it,
//! with its feet and its body on the drawn ground all the way; and a fleeing walker stops on the shore.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::fixtures::quadruped_with;
use super::rig::{Detail, Drive, Motion};
use super::tests_ground::{grounded, sunk};
use crate::fauna::shore::shore;

/// Seconds per step.
const DT: f32 = 1.0 / 30.0;

/// Speed of the flight in metres per second.
const FLIGHT: f32 = 8.0;

/// Metres from the robot within which the fleeing body is near, as `crate::fauna::tune::NEAR`.
const NEAR: f32 = 40.0;

/// Returns the height of a rolling, bumpy test terrain at `p`.
fn terrain(p: Vec2) -> f32 {
    let roll = 3.0 * (0.02 * p.x).sin() + 0.5 * (0.31 * p.x + 0.7).sin() * (0.23 * p.y).cos();
    roll + 0.15 * (1.7 * p.x).sin() * (1.3 * p.y + 0.4).cos()
}

/// Returns the height the terrain is drawn at over `p`: on the grid of the chunk ring or node there, 2 m
/// out to 64 m, then 4 m, 8 m, and 16 m past 192 m, two flat triangles a cell as the meshes split them.
fn drawn(p: Vec2) -> f32 {
    let step = match p.x {
        x if x < 64.0 => 2.0,
        x if x < 128.0 => 4.0,
        x if x < 192.0 => 8.0,
        _ => 16.0,
    };
    let cell = p / step;
    let (i, j) = (cell.x.floor(), cell.y.floor());
    let (u, v) = (cell.x - i, cell.y - j);
    let h = |a: f32, b: f32| terrain(Vec2::new(i + a, j + b) * step);
    if u + v <= 1.0 {
        h(0.0, 0.0) + u * (h(1.0, 0.0) - h(0.0, 0.0)) + v * (h(0.0, 1.0) - h(0.0, 0.0))
    } else {
        h(1.0, 1.0)
            + (1.0 - u) * (h(0.0, 1.0) - h(1.0, 1.0))
            + (1.0 - v) * (h(1.0, 0.0) - h(1.0, 1.0))
    }
}

/// Returns the drive of a fleeing body at `velocity` in `detail`.
fn fleeing(velocity: Vec3, detail: Detail) -> Drive {
    Drive {
        velocity,
        look: None,
        gravity: 9.81,
        excite: 1.0,
        detail,
    }
}

/// A walker fleeing at 8 m/s from a robot at the origin runs 200 m across every drawn level, near then
/// far, with no foot and no drawn part in the ground, and from a second after it is far from the robot
/// its body origin on the drawn ground within a centimetre.
#[test]
fn a_flight_over_every_drawn_level_stays_on_the_ground() {
    let body = quadruped_with(1, 0.4);
    let start = Vec3::new(0.0, drawn(Vec2::ZERO), 0.0);
    let mut m = Motion::new(&body, start, Vec3::X, 13);
    let (mut frame, mut far) = (0, 0);
    while m.root.at.x < 200.0 && frame < 1200 {
        let detail = if m.root.at.x < NEAR {
            Detail::Full
        } else {
            Detail::Cheap
        };
        m.step(&body, &fleeing(Vec3::X * FLIGHT, detail), &drawn, DT);
        let what = format!("{detail:?} flight at {:.1} m, frame {frame}", m.root.at.x);
        grounded(&m, &body, &drawn, &what);
        far = if detail == Detail::Cheap { far + 1 } else { 0 };
        if far > 30 {
            let off = sunk(&m, &drawn);
            assert!(
                off.abs() < 0.01,
                "{what}: body origin {off} m off the ground"
            );
        }
        frame += 1;
    }
    assert!(m.root.at.x >= 200.0, "fled only to {}", m.root.at.x);
}

/// A walker fleeing at full speed slantwise towards the sea runs along the shore and stops there: the
/// ground under it never goes 5 cm under the sea, while the part of its way along the shore is kept.
#[test]
fn a_fleeing_walker_stops_on_the_shore() {
    let body = quadruped_with(1, 0.4);
    let beach = |p: Vec2| 2.0 - 0.2 * p.x;
    let mut m = Motion::new(&body, Vec3::new(0.0, beach(Vec2::ZERO), 0.0), Vec3::X, 17);
    let way = Vec3::new(6.0, 0.0, 3.0);
    for frame in 0..300 {
        let velocity = shore(way, m.root.at, &body, &beach, 0.0);
        m.step(&body, &fleeing(velocity, Detail::Cheap), &beach, DT);
        let under = beach(Vec2::new(m.root.at.x, m.root.at.z));
        assert!(under > -0.05, "frame {frame}: ran {under} m into the sea");
    }
    assert!(
        m.root.at.z > 20.0,
        "stopped instead of running along the shore: {}",
        m.root.at
    );
    let dry = shore(way, m.root.at, &body, &beach, f32::NEG_INFINITY);
    assert_eq!(dry, way, "a world without a sea has no shore");
}

/// A walker fleeing at full speed straight at a lake 24 m wide, from three sides, stops on its shore:
/// the ground under it never goes 5 cm under the water. One already standing in the water as deep as
/// its hips wades at no more than a quarter of the speed it is asked for.
#[test]
fn a_fleeing_walker_stays_out_of_a_lake() {
    let body = quadruped_with(1, 0.4);
    let centre = Vec2::new(30.0, 0.0);
    let lake = |p: Vec2| 1.5 - 3.0 * (-(p - centre).length_squared() / 200.0).exp();
    for side in [Vec2::NEG_X, Vec2::Y, Vec2::new(-1.0, -1.0).normalize()] {
        let start = centre + side * 30.0;
        let at = Vec3::new(start.x, lake(start), start.y);
        let mut m = Motion::new(&body, at, (-side).extend(0.0).xzy(), 19);
        let way = (-side * 8.0).extend(0.0).xzy();
        for frame in 0..240 {
            let velocity = shore(way, m.root.at, &body, &lake, 0.0);
            m.step(&body, &fleeing(velocity, Detail::Full), &lake, DT);
            let under = lake(Vec2::new(m.root.at.x, m.root.at.z));
            assert!(
                under > -0.05,
                "from {side}, frame {frame}: ran {under} m into the lake"
            );
        }
    }
    let wading = shore(
        Vec3::X * 6.0,
        Vec3::ZERO,
        &body,
        &|_: Vec2| -body.height,
        0.0,
    );
    assert!(
        wading.length() <= 1.5 + 1e-4,
        "waded at {} m/s",
        wading.length()
    );
}
