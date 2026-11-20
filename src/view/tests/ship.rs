//! The ship: a cockpit that lags a little, and a chase camera that shows the whole hull.

use bevy::prelude::*;

use super::world::Boxes;
use super::{first, moment, run, seen, third};
use crate::view::tune::RADIUS;
use crate::view::{Kind, Moment, Rig, Shown, ViewPrefs};

/// Returns a ship of `radius` metres seen with `prefs`, at rest in open space.
fn ship(radius: f32, prefs: ViewPrefs) -> Moment {
    let mut ship = moment(Kind::Ship, prefs);
    ship.target.radius = radius;
    ship
}

/// Returns true when the sphere of `radius` around `centre` is wholly inside the view of
/// `shown` on a 16 by 9 screen.
fn in_view(shown: &Shown, centre: Vec3, radius: f32) -> bool {
    let eye = shown.pose.rotation.inverse() * (centre - shown.pose.position);
    let tall = 0.5 * shown.pose.fov;
    let wide = (tall.tan() * 16.0 / 9.0).atan();
    let room = |half: f32, across: f32| -eye.z * half.sin() - across.abs() * half.cos();
    room(tall, eye.y) >= radius && room(wide, eye.x) >= radius
}

/// The chase distance is the size of the ship times a constant, and the whole ship is in
/// view at the default settings: at rest, at speed in an atmosphere and during a warp.
#[test]
fn chase_scales_with_the_ship_and_shows_all_of_it() {
    let world = Boxes::default();
    let rest = |radius: f32| {
        let mut ship = ship(radius, third());
        let shown = run(&mut Rig::default(), &mut ship, &world, 1);
        assert!(in_view(&shown, ship.target.position, radius));
        seen(&shown, &ship.target).0
    };
    let (small, large) = (rest(4.5), rest(18.0));
    assert!(small.z > 0.0 && small.y > 0.0 && small.distance(large / 4.0) < 1.0e-4);
    for velocity in [Vec3::new(0.0, 20.0, -200.0), Vec3::NEG_Z * 4.0e6] {
        let mut ship = ship(4.5, third());
        ship.target.velocity = velocity;
        ship.target.ground = Vec3::Y * 0.8;
        let shown = run(&mut Rig::default(), &mut ship, &world, 300);
        assert!(in_view(&shown, ship.target.position, 4.5));
    }
}

/// The chase camera pulls back and widens with speed, and more during a warp.
#[test]
fn chase_pulls_back_and_widens_with_speed() {
    let view = |speed: f32| {
        let mut ship = ship(4.5, third());
        ship.target.velocity = Vec3::NEG_Z * speed;
        let shown = run(&mut Rig::default(), &mut ship, &Boxes::default(), 300);
        (seen(&shown, &ship.target).0.length(), shown.pose.fov)
    };
    let (rest, cruise, warp) = (view(0.0), view(400.0), view(4.0e6));
    assert!(cruise.0 > rest.0 * 1.1 && warp.0 > cruise.0 * 1.1);
    assert!(cruise.1 > rest.1 + 0.05 && warp.1 > cruise.1 + 0.05);
}

/// In space the chase camera keeps the up of the ship; low over a planet it levels toward
/// the local up, and at speed it turns toward where the ship is going.
#[test]
fn chase_keeps_the_ship_up_in_space_and_levels_near_a_planet() {
    let world = Boxes::default();
    let mut ship = ship(4.5, third());
    ship.target.up = Vec3::X;
    let space = run(&mut Rig::default(), &mut ship, &world, 120);
    assert!((space.pose.rotation * Vec3::Y).distance(Vec3::X) < 1.0e-4);
    ship.target.ground = Vec3::Y;
    let low = run(&mut Rig::default(), &mut ship, &world, 240);
    assert!((low.pose.rotation * Vec3::Y).y > 0.9);
    ship.target.velocity = Vec3::new(120.0, 0.0, -120.0);
    let fast = run(&mut Rig::default(), &mut ship, &world, 240);
    let (nose, ahead) = (
        low.pose.rotation * Vec3::NEG_Z,
        fast.pose.rotation * Vec3::NEG_Z,
    );
    assert!(nose.x.abs() < 1.0e-3 && ahead.x > 0.2 && ahead.x < 0.6);
}

/// The mouse looks around the ship, the view holds for a moment once the mouse stops, and
/// then the camera springs back behind the ship.
#[test]
fn the_mouse_looks_around_and_the_view_springs_back() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut ship = ship(4.5, third());
    ship.look.yaw_delta = 0.03;
    let orbit = run(&mut rig, &mut ship, &world, 30);
    assert!((orbit.heading.yaw - 0.9).abs() < 1.0e-4);
    assert!(seen(&orbit, &ship.target).0.x > 4.0);
    ship.look.yaw_delta = 0.0;
    let held = run(&mut rig, &mut ship, &world, 60);
    assert!((held.heading.yaw - 0.9).abs() < 1.0e-4);
    let back = run(&mut rig, &mut ship, &world, 400);
    assert!(back.heading.yaw.abs() < 1.0e-3 && seen(&back, &ship.target).0.x.abs() < 0.05);
}

/// The cockpit sits at the pilot seat and trails a turning ship by a small angle.
#[test]
fn cockpit_lags_behind_the_rotation() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut ship = ship(4.5, first());
    let still = run(&mut rig, &mut ship, &world, 1);
    assert!(
        seen(&still, &ship.target)
            .0
            .distance(Vec3::new(0.0, 0.9, -1.305))
            < 1.0e-4
    );
    ship.target.spin = Vec3::Y;
    let mut behind = 0.0;
    for frame in 1..=120 {
        ship.target.forward = Quat::from_rotation_y(frame as f32 * ship.dt) * Vec3::NEG_Z;
        let shown = run(&mut rig, &mut ship, &world, 1);
        behind = (shown.pose.rotation * Vec3::NEG_Z).angle_between(ship.target.forward);
        assert!(behind < 0.1);
    }
    assert!(behind > 0.05, "the cockpit does not lag: {behind}");
}

/// A landed ship seen from behind never has the camera in the ground, wherever the player orbits.
#[test]
fn chase_never_clips_the_ground() {
    let world = Boxes::floor().with([-30.0, 0.0, 9.0], [30.0, 20.0, 10.0]);
    let (mut rig, mut ship) = (Rig::default(), ship(4.5, third()));
    ship.target.position = Vec3::Y * 1.2;
    ship.target.ground = Vec3::Y;
    for frame in 0..600 {
        ship.look.yaw_delta = 0.02;
        ship.look.pitch_delta = 0.03 * (frame as f32 * 0.02).sin();
        let at = run(&mut rig, &mut ship, &world, 1).pose.position;
        assert!(
            world.clearance(at) >= RADIUS - 1.0e-3,
            "the camera is in the ground at {at}"
        );
    }
}
