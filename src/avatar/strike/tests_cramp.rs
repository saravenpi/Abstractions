//! Tests of a held tool kept out of a wall: walking into a wall with each tool drawn, no point of
//! the arm's reach or of the blade is inside it on any frame, and stepping back brings the tool down
//! to where it is carried.

use bevy::prelude::*;

use super::cramp::{ease, need, rest};
use super::stage::{Plane, blade, club, knife, spear};

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// Returns how deep into `wall` the deepest point along the shoulder, the grip and the tip of
/// `chain` is: 0 when none is.
fn inside(wall: &Plane, [shoulder, grip, tip]: [Vec3; 3]) -> f32 {
    (0..=10)
        .flat_map(|n| {
            let s = n as f32 / 10.0;
            [shoulder.lerp(grip, s), grip.lerp(tip, s)]
        })
        .map(|p| (p - wall.at).dot(wall.normal))
        .fold(0.0, |most: f32, d| most.max(-d))
}

/// Returns the wall `metres` ahead of the body, facing it.
fn wall(metres: f32) -> Plane {
    Plane {
        at: Vec3::new(0.0, 0.0, -metres),
        normal: Vec3::Z,
    }
}

/// Walking at 1.5 m/s from 2 m to 25 cm from a wall and back with a knife, a blade, a spear and a
/// club drawn, the tool rises as far as keeps it out of the wall on every frame, and comes back down
/// to the carried pose once the wall is far.
#[test]
fn a_held_tool_stays_out_of_a_wall() {
    for held in [knife(), blade(), spear(), club()] {
        let reach = held.build.reach();
        let (mut raise, mut most): (f32, f32) = (0.0, 0.0);
        let mut ahead = 2.0_f32;
        for frame in 0..200 {
            let towards = if frame < 70 { -1.5 } else { 1.5 };
            ahead = (ahead + towards * FRAME).clamp(0.25, 2.0);
            let solid = wall(ahead);
            raise = ease(raise, need(&solid, |p| p, reach), FRAME);
            let deep = inside(&solid, rest(raise, reach));
            assert!(
                deep < 1e-3,
                "reach {reach}, frame {frame}, wall at {ahead}: {deep} in it"
            );
            most = most.max(raise);
        }
        assert!(most > 0.3, "reach {reach} never rose: {most}");
        assert_eq!(raise, 0.0, "reach {reach} stayed raised");
        assert_eq!(rest(raise, reach), rest(0.0, reach));
    }
}
