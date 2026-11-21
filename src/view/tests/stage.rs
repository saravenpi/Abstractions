//! A ship with a camera parked around it: the shared setup of the tests that hold the view.

use bevy::prelude::*;

use super::world::Boxes;
use super::{frame, moment, run, seen, third};
use crate::view::{Heading, Kind, Look, Moment, Rig, Shown, Target, ViewPrefs};

/// The camera as the player parked it behind the ship.
#[derive(Debug, Clone, Copy)]
pub(super) struct Parked {
    /// The look angles.
    pub heading: Heading,
    /// The direction from the centre of the hull to the camera and the turn of the camera,
    /// both in the axes of the hull.
    pub angle: (Vec3, Quat),
    /// How far the camera is from the centre of the hull, in metres.
    pub reach: f32,
}

/// Returns a ship of 4.5 m at rest in open space, seen from behind, with auto-follow `follow`.
pub(super) fn ship(follow: bool) -> Moment {
    let prefs = ViewPrefs {
        auto_follow: follow,
        ..third()
    };
    let mut ship = moment(Kind::Ship, prefs);
    ship.target.radius = 4.5;
    ship.target.position = Vec3::Y * 40.0;
    ship
}

/// Returns where the camera of `shown` sits around `target`: the direction from its centre to
/// the camera and the turn of the camera, both in the axes of the target.
pub(super) fn angle(shown: &Shown, target: &Target) -> (Vec3, Quat) {
    let (offset, turn) = seen(shown, target);
    (offset.normalize(), turn)
}

/// Returns a rig behind a ship with auto-follow off, after the mouse turned the look to
/// `heading` in half a second and then rested for two seconds, and the camera as parked then.
pub(super) fn parked(heading: Heading) -> (Rig, Moment, Parked) {
    let (mut rig, mut ship, world) = (Rig::default(), ship(false), Boxes::default());
    ship.look.yaw_delta = heading.yaw / 30.0;
    ship.look.pitch_delta = heading.pitch / 30.0;
    run(&mut rig, &mut ship, &world, 30);
    ship.look = Look::default();
    let shown = run(&mut rig, &mut ship, &world, 120);
    let parked = Parked {
        heading: shown.heading,
        angle: angle(&shown, &ship.target),
        reach: seen(&shown, &ship.target).0.length(),
    };
    (rig, ship, parked)
}

/// Turns the hull of `ship` for one frame at `rate`, in radians per second about the axes of
/// the hull, and gives the rig that angular velocity in render axes.
pub(super) fn turn(ship: &mut Moment, rate: Vec3) {
    let hull = (frame(&ship.target) * Quat::from_scaled_axis(rate * ship.dt)).normalize();
    ship.target.forward = hull * Vec3::NEG_Z;
    ship.target.up = hull * Vec3::Y;
    ship.target.spin = hull * rate;
}

/// Moves render space under `target` as a rebase does: turned by `turn` about the origin, then
/// shifted by `shift`.
pub(super) fn moved(target: &mut Target, turn: Quat, shift: Vec3) {
    target.position = turn * target.position + shift;
    target.up = turn * target.up;
    target.forward = turn * target.forward;
    target.velocity = turn * target.velocity;
    target.spin = turn * target.spin;
    target.ground = turn * target.ground;
}

/// Flies `ship` for `frames` frames, each first set up by `each` at its time in seconds, and
/// checks on every frame that the look stays where it was `parked`.
pub(super) fn fly(
    rig: &mut Rig,
    ship: &mut Moment,
    parked: &Parked,
    frames: usize,
    each: impl Fn(&mut Moment, f32),
) {
    for step in 0..frames {
        let time = step as f32 * ship.dt;
        each(ship, time);
        let shown = run(rig, ship, &Boxes::default(), 1);
        assert_eq!(shown.heading, parked.heading, "the look moved at {time} s");
    }
}
