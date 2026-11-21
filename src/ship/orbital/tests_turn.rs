//! Tests of the turn onto the course of a cruise: from the press of the home key the ship turns
//! at a bounded rate, easing in and out, onto the course within a few seconds, builds its speed
//! only once its nose is on it, and keeps its nose at the handover to the glide.

use bevy::math::{DQuat, DVec3};

use super::carry::advance;
use super::near::Over;
use super::rig::{DT, band, earth};
use super::skim::{Ask, attitude};
use super::tune::{ALIGNED, RATE, SLEW};
use super::{Orbital, Regime};
use crate::ship::body::Ship;
use crate::space::Frame;
use crate::universe::PlanetSpec;

/// Seconds the turn onto a course behind the ship may take: half a turn at 40 degrees a second,
/// with the easing in and out.
const TURNED: f64 = 6.0;

/// Returns the planet still at the origin of its own axes.
fn still(planet: &PlanetSpec) -> Over {
    Over {
        planet: planet.clone(),
        centre: DVec3::ZERO,
        vel: DVec3::ZERO,
        turn: DQuat::IDENTITY,
        spin: DVec3::ZERO,
    }
}

/// Flies one frame of the cruise and returns how far the ship turned in it, checked to stay under
/// 40 degrees a second, with the sweep of the cruise, and to change smoothly from the swing
/// `last` of the frame before.
fn swung(orbital: &mut Orbital, ship: &mut Ship, over: &Over, asked: &Ask, last: f64) -> f64 {
    let before = ship.rot;
    advance(orbital, ship, over, asked, DT).expect("still cruising");
    let swing = before.angle_between(ship.rot);
    assert!(
        swing <= (SLEW.0 + RATE) * DT + 1e-6,
        "{swing} rad in a frame"
    );
    let change = (swing - last).abs();
    assert!(
        change <= 2.0 * SLEW.1 * DT * DT + RATE * DT + 1e-6,
        "the turn jumped by {change}"
    );
    swing
}

/// Returns the angle of the nose of `ship` off the course to +Z, and the same angle along the
/// ground.
fn off(ship: &Ship) -> (f64, f64) {
    let up = ship.pos.normalize();
    let course = (DVec3::Z - up * up.z).normalize();
    let nose = ship.forward();
    let flat = (nose - up * nose.dot(up)).normalize();
    (nose.angle_between(course), flat.angle_between(course))
}

/// From a nose pointing away from home and pitched up, the home key turns the ship by at most
/// 40 degrees a second onto the course, the turn never jumping from one frame to the next, the
/// speed staying at nothing until the nose is within five degrees and building after.
#[test]
fn the_ship_turns_onto_the_course_before_it_speeds_up() {
    let planet = earth();
    let over = still(&planet);
    let pos = DVec3::X * (planet.radius + band(&planet).floor);
    let rot = attitude(DVec3::X, DVec3::NEG_Z, 0.3);
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), pos, rot);
    let mut orbital = Orbital::default();
    orbital.start(&mut ship, &over, Some(DVec3::Z * planet.radius));
    let asked = Ask {
        throttle: 1.0,
        goal: Some(DVec3::Z),
        ..Ask::default()
    };
    let (mut time, mut turned, mut last, mut on) = (0.0, None, 0.0f64, false);
    while time < 3.0 * TURNED {
        last = swung(&mut orbital, &mut ship, &over, &asked, last);
        let (off, flat) = off(&ship);
        on |= flat < ALIGNED;
        assert!(
            on || orbital.speed == 0.0,
            "sped up {flat} rad off the course"
        );
        if off < 0.02 && turned.is_none() {
            turned = Some(time);
        }
        time += DT;
    }
    let turned = turned.expect("the nose never came onto the course");
    assert!(turned < TURNED, "the turn took {turned} s");
    assert!(orbital.speed > 100_000.0, "only {} m/s", orbital.speed);
}

/// At the handover to the glide the nose stays where it was: the glide turns it down.
#[test]
fn the_handover_to_the_glide_keeps_the_nose() {
    let planet = earth();
    let over = still(&planet);
    let pos = DVec3::X * (planet.radius + band(&planet).floor);
    let rot = attitude(DVec3::X, DVec3::NEG_Z, 0.0);
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), pos, rot);
    let mut orbital = Orbital::default();
    orbital.start(&mut ship, &over, Some(DVec3::NEG_Z * planet.radius));
    orbital.descend(&mut ship, &over);
    assert_eq!(orbital.regime(), Regime::Glide);
    assert_eq!(ship.rot, rot);
    assert!(
        ship.vel.dot(DVec3::NEG_Z) > 0.0,
        "the glide does not go on along the course"
    );
}
