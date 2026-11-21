//! Holding the view in the ship: with auto-follow off, the angle the player left the camera at
//! around the ship stays through flight, a frame change, rebases, occlusion and the cockpit.

use bevy::prelude::*;

use super::stage::{Parked, angle, fly, moved, parked, turn};
use super::world::Boxes;
use super::{apart, run, seen};
use crate::view::tune::COCKPIT_TURN;
use crate::view::{Heading, Moment, Rig, ViewMode};

/// Gives `ship` the velocity of a fast flight along its nose, slipping sideways.
fn slip(ship: &mut Moment) {
    let right = ship.target.forward.cross(ship.target.up);
    ship.target.velocity = ship.target.forward * 200.0 + right * 50.0;
}

/// Returns how far the camera of the next frame is from where it was `parked` around the
/// hull: the gap between the directions, and the angle between the turns of the camera.
fn astray(rig: &mut Rig, ship: &mut Moment, parked: &Parked) -> (f32, f32) {
    let shown = run(rig, ship, &Boxes::default(), 1);
    apart(parked.angle, angle(&shown, &ship.target))
}

/// With auto-follow off, a chase angle left alone holds through 30 s of flight with turns:
/// 15 s low and fast, slipping sideways through yaw turns, where the chase camera does not
/// turn toward the path, then 15 s in space through turns about every axis. The look never
/// moves, and once the ship flies straight the camera is back where it was parked on the hull.
#[test]
fn a_parked_chase_angle_survives_a_flight_with_turns() {
    let (mut rig, mut ship, rest) = parked(Heading {
        yaw: 0.9,
        pitch: -0.3,
    });
    let low = |ship: &mut Moment, time: f32| {
        ship.target.ground = ship.target.up * 0.8;
        turn(ship, Vec3::Y * 0.6 * (0.7 * time).sin());
        slip(ship);
    };
    fly(&mut rig, &mut ship, &rest, 900, low);
    fly(&mut rig, &mut ship, &rest, 240, |ship, _| low(ship, 0.0));
    let (_, turned) = astray(&mut rig, &mut ship, &rest);
    assert!(
        turned < 1.0e-3,
        "the chase turned by {turned} rad toward the path"
    );
    let axis = Vec3::new(0.3, 1.0, 0.5).normalize();
    let space = |ship: &mut Moment, time: f32| {
        ship.target.ground = Vec3::ZERO;
        turn(ship, axis * 0.5 * (0.5 * time).sin());
        slip(ship);
    };
    fly(&mut rig, &mut ship, &rest, 900, space);
    fly(&mut rig, &mut ship, &rest, 240, |ship, _| space(ship, 0.0));
    let (gap, turned) = astray(&mut rig, &mut ship, &rest);
    assert!(
        gap < 1.0e-3 && turned < 1.0e-3,
        "{gap} and {turned} rad off"
    );
}

/// With auto-follow off, the parked angle holds when the ship climbs out of the air into the
/// frame of its star, where the ground goes, render space turns and moves and the speed is
/// measured against new surroundings, and while the anchor moves under the ship every frame.
#[test]
fn a_parked_chase_angle_survives_a_frame_change_and_rebases() {
    let (mut rig, mut ship, rest) = parked(Heading {
        yaw: -2.4,
        pitch: 0.35,
    });
    let climb = |ship: &mut Moment, _: f32| {
        ship.target.ground = ship.target.up * 0.3;
        ship.target.velocity = ship.target.forward * 150.0;
    };
    fly(&mut rig, &mut ship, &rest, 120, climb);
    let turned = Quat::from_axis_angle(Vec3::new(0.2, 0.7, -0.4).normalize(), 1.1);
    moved(&mut ship.target, turned, Vec3::new(-300.0, 50.0, 800.0));
    ship.target.ground = Vec3::ZERO;
    ship.target.velocity = ship.target.forward * 600.0;
    let creep = Quat::from_rotation_y(2.0e-4);
    let moor =
        |ship: &mut Moment, _: f32| moved(&mut ship.target, creep, Vec3::new(0.4, -0.1, 0.7));
    fly(&mut rig, &mut ship, &rest, 240, moor);
    let (gap, turned) = astray(&mut rig, &mut ship, &rest);
    assert!(
        gap < 1.0e-3 && turned < 1.0e-3,
        "{gap} and {turned} rad off"
    );
}

/// With auto-follow off, terrain between the ship and a parked camera pulls the camera in
/// along the same line without turning it, and once the terrain is gone the camera goes back
/// out to where it was parked.
#[test]
fn occlusion_pulls_the_parked_camera_in_without_turning_it() {
    let (mut rig, mut ship, rest) = parked(Heading {
        yaw: 0.9,
        pitch: -0.3,
    });
    let open = Boxes::default();
    let at = run(&mut rig, &mut ship, &open, 1).pose.position;
    let middle = ship.target.position.lerp(at, 0.6);
    let low = (middle - Vec3::splat(2.0)).to_array();
    let hill = open
        .clone()
        .with(low, (middle + Vec3::splat(2.0)).to_array());
    let blocked = run(&mut rig, &mut ship, &hill, 30);
    let reach = seen(&blocked, &ship.target).0.length();
    let (gap, turned) = apart(rest.angle, angle(&blocked, &ship.target));
    assert!(
        reach < rest.reach - 3.0,
        "the hill did not pull the camera in: {reach} m"
    );
    assert!(
        gap < 1.0e-4 && turned < 1.0e-4,
        "{gap} and {turned} rad off"
    );
    assert_eq!(blocked.heading, rest.heading);
    let clear = run(&mut rig, &mut ship, &open, 600);
    let reach = seen(&clear, &ship.target).0.length();
    let (gap, turned) = apart(rest.angle, angle(&clear, &ship.target));
    assert!((reach - rest.reach).abs() < 0.01 && gap < 1.0e-4 && turned < 1.0e-4);
}

/// With auto-follow off, a chase angle beyond the reach of the head in the pilot seat comes
/// back whole after a switch to the cockpit and back: in the seat the view stops at the reach
/// of the head, and behind the ship again the camera is where it was parked.
#[test]
fn the_cockpit_and_back_keeps_the_parked_chase_angle() {
    let (mut rig, mut ship, rest) = parked(Heading {
        yaw: 2.6,
        pitch: -0.4,
    });
    let world = Boxes::default();
    ship.prefs.ship = ViewMode::First;
    let seated = run(&mut rig, &mut ship, &world, 120);
    assert!(seated.heading.yaw <= COCKPIT_TURN + 1.0e-5 && seated.fade.0 == 0.0);
    ship.prefs.ship = ViewMode::Third;
    let back = run(&mut rig, &mut ship, &world, 120);
    let yaw = (back.heading.yaw - rest.heading.yaw).abs();
    let pitch = (back.heading.pitch - rest.heading.pitch).abs();
    assert!(yaw < 1.0e-5 && pitch < 1.0e-5, "{:?}", back.heading);
    let (gap, turned) = apart(rest.angle, angle(&back, &ship.target));
    assert!(
        gap < 1.0e-4 && turned < 1.0e-4,
        "{gap} and {turned} rad off"
    );
}

/// In the pilot seat the mouse turns the head from where the view shows: a parked chase angle
/// beyond the reach of the head is no dead zone the mouse has to wind back first.
#[test]
fn the_mouse_turns_the_seated_view_from_where_it_shows() {
    let (mut rig, mut ship, _) = parked(Heading {
        yaw: 2.6,
        pitch: 0.0,
    });
    let world = Boxes::default();
    ship.prefs.ship = ViewMode::First;
    run(&mut rig, &mut ship, &world, 60);
    ship.look.yaw_delta = -0.05;
    let turned = run(&mut rig, &mut ship, &world, 1);
    assert!((turned.heading.yaw - (COCKPIT_TURN - 0.05)).abs() < 1.0e-4);
}
