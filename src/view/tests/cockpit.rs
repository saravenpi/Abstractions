//! The pilot seat: the eyes turn on the neck inside the canopy, the head stops before the
//! headrest, and the mouse looks around only while it does not steer.

use bevy::prelude::*;

use super::world::Boxes;
use super::{apart, moment, run, seen, third, first};
use crate::view::keys::free;
use crate::view::tune::{COCKPIT_TURN, COCKPIT_UP};
use crate::view::{FACE, Kind, Moment, Rig, ViewPrefs};

/// The neck of the pilot in the axes of a ship of 4.5 m: the pivot of the first person view.
const NECK: Vec3 = Vec3::new(0.0, 0.68, -1.145);

/// Returns a ship of 4.5 m seen with `prefs`, at rest in open space.
fn ship(prefs: ViewPrefs) -> Moment {
    let mut ship = moment(Kind::Ship, prefs);
    ship.target.radius = 4.5;
    ship
}

/// With the mouse the pilot turns the head: the eyes swing round the neck, forward over the
/// chest when they look down, and the body is drawn without its head. From behind the ship
/// it is drawn whole.
#[test]
fn the_pilot_looks_around_from_the_neck() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut pilot = ship(first());
    let rest = run(&mut rig, &mut pilot, &world, 1);
    assert_eq!(rest.fade.0, 0.0);
    assert!(seen(&rest, &pilot.target).0.distance(NECK + FACE) < 1.0e-4);
    pilot.look.yaw_delta = 0.02;
    let left = run(&mut rig, &mut pilot, &world, 50);
    let (eye, turn) = seen(&left, &pilot.target);
    assert!((left.heading.yaw - 1.0).abs() < 1.0e-4 && (turn * Vec3::NEG_Z).x < -0.8);
    assert!(((eye - NECK).length() - FACE.length()).abs() < 1.0e-4 && eye.x < -0.1);
    (pilot.look.yaw_delta, pilot.look.pitch_delta) = (-0.02, -0.02);
    let down = run(&mut rig, &mut pilot, &world, 50);
    let eye = seen(&down, &pilot.target).0;
    assert!(eye.z < NECK.z - 0.2 && eye.y < NECK.y + 0.05 && down.fade.0 == 0.0);
    let chase = run(&mut Rig::default(), &mut ship(third()), &world, 1);
    assert_eq!(chase.fade.0, 1.0);
}

/// However far the mouse turns, the head of the pilot stops at its reach to each side and
/// upward, and the eyes never swing back into the headrest.
#[test]
fn the_head_stops_before_the_headrest() {
    let world = Boxes::default();
    for (yaw, pitch) in [
        (0.05, 0.0),
        (-0.05, 0.0),
        (0.0, 0.05),
        (0.05, 0.05),
        (0.0, -0.05),
    ] {
        let (mut rig, mut pilot) = (Rig::default(), ship(first()));
        (pilot.look.yaw_delta, pilot.look.pitch_delta) = (yaw, pitch);
        for _ in 0..200 {
            let shown = run(&mut rig, &mut pilot, &world, 1);
            let eye = seen(&shown, &pilot.target).0;
            assert!(shown.heading.yaw.abs() <= COCKPIT_TURN + 1.0e-5);
            assert!(shown.heading.pitch <= COCKPIT_UP + 1.0e-5);
            assert!(eye.z < -1.05, "the eyes are in the headrest at {eye}");
        }
    }
}

/// The chase camera orbits all the way round, and a switch to the seat from there brings the
/// view inside the reach of the head without a jump.
#[test]
fn the_orbit_is_free_and_folds_into_the_seat_smoothly() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut pilot = ship(third());
    pilot.look.yaw_delta = 0.05;
    let behind = run(&mut rig, &mut pilot, &world, 60);
    assert!((behind.heading.yaw - 3.0).abs() < 1.0e-3);
    pilot.look.yaw_delta = 0.0;
    pilot.prefs = first();
    let mut last = seen(&behind, &pilot.target);
    for _ in 0..40 {
        let shown = run(&mut rig, &mut pilot, &world, 1);
        let now = seen(&shown, &pilot.target);
        let (moved, turned) = apart(last, now);
        assert!(
            moved < 1.5 && turned < 0.2,
            "a jump of {moved} m, {turned} rad"
        );
        last = now;
    }
    let seated = run(&mut rig, &mut pilot, &world, 1);
    assert!(seated.heading.yaw <= COCKPIT_TURN + 1.0e-5 && seated.fade.0 == 0.0);
}

/// In the ship the mouse looks around while the steering button is up and leaves the view
/// alone while it is held; on foot and on the wings the view always takes the mouse.
#[test]
fn the_held_button_keeps_the_view_still_in_the_ship() {
    let motion = Vec2::new(12.0, -7.0);
    assert_eq!(free(motion, Kind::Ship, false), motion);
    assert_eq!(free(motion, Kind::Ship, true), Vec2::ZERO);
    for kind in [Kind::OnFoot, Kind::Flying] {
        assert_eq!(
            (free(motion, kind, true), free(motion, kind, false)),
            (motion, motion)
        );
    }
}
