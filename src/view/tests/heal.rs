//! The camera through a warp between galaxies and a change of frame: a number on every frame,
//! with the roll and the level of its rule, and back behind the ship at its distance and field
//! of view within a second of the view switching back.

use bevy::prelude::*;

use super::stage::{Parked, moved, parked};
use super::world::Boxes;
use super::{run, seen};
use crate::view::tune::MAX_FOV;
use crate::view::{Heading, Moment, Rig, Shown, ViewMode};

/// Speed of a warp between galaxies, in metres per second: 3600 light years a second. Its
/// square is past the range of f32.
const GALACTIC: f32 = 3.4e19;

/// Returns the camera a rig at rest shows on `ship` with the look `heading`: the roll and the
/// level the camera eases to. In space it rolls with the ship; near a body it levels.
fn rule(ship: &Moment, heading: Heading) -> Shown {
    let mut fresh = Rig::default();
    fresh.face(heading);
    fresh.step(ship, &Boxes::default())
}

/// Returns the angle between the turn of `shown` and that of its rule on `ship`.
fn off_rule(shown: &Shown, ship: &Moment, heading: Heading) -> f32 {
    shown
        .pose
        .rotation
        .angle_between(rule(ship, heading).pose.rotation)
}

/// Warps `ship` for two seconds at the speed between galaxies and changes galaxy halfway: render
/// space jumps to 1e22 m for one frame and comes back turned. Then the warp ends. Checks on
/// every frame that the camera is a number and has the roll and the level of its rule.
fn cross(rig: &mut Rig, ship: &mut Moment, heading: Heading) {
    let (world, home) = (Boxes::default(), ship.target.position);
    let turn = Quat::from_axis_angle(Vec3::new(0.4, -0.2, 0.9).normalize(), 2.3);
    for frame in 0..120 {
        ship.target.velocity = ship.target.forward * GALACTIC;
        match frame {
            60 => ship.target.position = Vec3::splat(1.0e22),
            61 => {
                ship.target.position = home;
                moved(&mut ship.target, turn, Vec3::ZERO);
            }
            _ => {}
        }
        let shown = rig.step(ship, &world);
        assert!(shown.sound(), "frame {frame}: {shown:?}");
        let off = off_rule(&shown, ship, heading);
        assert!(off < 2.0e-3, "frame {frame}: {off} rad off its rule");
    }
    ship.target.velocity = Vec3::ZERO;
}

/// Checks that one second after the view switched back behind the ship, and for good five
/// seconds later, the camera is at its distance, field of view and roll around the hull.
fn behind_again(rig: &mut Rig, ship: &mut Moment, rest: &Parked) {
    let world = Boxes::default();
    ship.prefs.ship = ViewMode::Third;
    for (frames, slack) in [(60, 0.15), (300, 0.01)] {
        let shown = run(rig, ship, &world, frames);
        let goal = rule(ship, rest.heading);
        let reach = seen(&shown, &ship.target).0.length() / rest.reach;
        let wide = (shown.pose.fov - goal.pose.fov).abs();
        assert!(
            reach > 1.0 - slack && reach < 1.0 + slack,
            "{reach} of the distance"
        );
        assert!(
            wide < slack && shown.pose.fov <= MAX_FOV,
            "{wide} rad wider"
        );
        let off = off_rule(&shown, ship, rest.heading);
        assert!(off < slack.min(0.01), "{off} rad off its rule");
    }
}

/// From behind the ship, a warp and a change of galaxy, then a switch to the seat and back:
/// the chase camera is behind the ship again, not inside the hull.
#[test]
fn the_chase_camera_comes_back_after_a_galaxy_change() {
    let heading = Heading {
        yaw: 0.5,
        pitch: -0.2,
    };
    let (mut rig, mut ship, rest) = parked(heading);
    cross(&mut rig, &mut ship, rest.heading);
    ship.prefs.ship = ViewMode::First;
    run(&mut rig, &mut ship, &Boxes::default(), 30);
    behind_again(&mut rig, &mut ship, &rest);
}

/// From the seat, a warp and a change of galaxy, then a switch behind the ship: the chase camera
/// is behind the ship at its distance.
#[test]
fn the_seat_switches_behind_the_ship_after_a_galaxy_change() {
    let heading = Heading {
        yaw: 0.5,
        pitch: -0.2,
    };
    let (mut rig, mut ship, rest) = parked(heading);
    ship.prefs.ship = ViewMode::First;
    run(&mut rig, &mut ship, &Boxes::default(), 60);
    cross(&mut rig, &mut ship, rest.heading);
    behind_again(&mut rig, &mut ship, &rest);
}

/// A ship that flies rolled in space keeps the camera rolled with it. Once it is low over a
/// planet the camera levels toward the local up within a second, and keeps no tilt of its own.
#[test]
fn the_roll_eases_to_its_rule_after_entering_a_planet() {
    let (mut rig, mut ship, rest) = parked(Heading::default());
    let world = Boxes::default();
    ship.target.up = Quat::from_rotation_z(0.6) * Vec3::Y;
    let space = run(&mut rig, &mut ship, &world, 120);
    assert!(off_rule(&space, &ship, rest.heading) < 2.0e-3);
    let rolled = (space.pose.rotation * Vec3::Y).y;
    ship.target.ground = Vec3::Y;
    let low = run(&mut rig, &mut ship, &world, 60);
    assert!(off_rule(&low, &ship, rest.heading) < 0.01);
    assert!(
        (low.pose.rotation * Vec3::Y).y > rolled + 0.05,
        "the camera did not level"
    );
}
