//! Where the footsteps come from and what carries them: a step packs into one word, the steps
//! of others come from where they are, late and quieter, in a vacuum only the listener's own
//! step is heard, through the body, and the armour of the robot ticks on a hard landing.

use super::RATE;
use super::rig::{band, rms};
use super::takes_steps::heard;
use super::treads::{left, trod, trod_at};
use crate::audio::cue::StepSurface;
use crate::audio::spatial::Air;
use crate::audio::step::{Contact, Step};

/// A step survives the trip through one word, each field within half of its unit.
#[test]
fn a_step_packs_into_a_word() {
    let step = Step {
        contact: Contact::Landing,
        force: 0.7,
        load: 9.3,
        mass: 25.8,
        roll: 0.031,
        lift: 0.3,
        depth: 0.21,
        left: true,
        cold: true,
        ..Step::on(StepSurface::Hull)
    };
    let back = Step::unpack(step.pack()).expect("a step");
    assert_eq!(
        (back.surface, back.contact, back.left, back.cold),
        (step.surface, step.contact, step.left, step.cold)
    );
    let near = |a: f32, b: f32, unit: f32| (a - b).abs() <= 0.5 * unit + 1e-5;
    assert!(near(back.force, step.force, 1.0 / 63.0) && near(back.load, step.load, 0.25));
    assert!(near(back.mass, step.mass, 4.0) && near(back.roll, step.roll, 0.008));
    assert!(near(back.lift, step.lift, 0.01) && near(back.depth, step.depth, 0.04));
}

/// The step of someone 10 metres off on the right arrives a sound's travel late, louder on the
/// right, and as `1 / r` quieter than the listener's own; far enough off it falls under the
/// floor of hearing and is not played at all.
#[test]
fn other_steps_come_from_where_they_are() {
    let rock = Step::on(StepSurface::Rock);
    let home = Air::home();
    let spot = heard(10.0, 0.8, home);
    let (own, far) = (trod(rock, 1), trod_at(rock, spot, 1));
    let start = far.iter().position(|frame| frame[1].abs() > 1.0e-6);
    let travel = spot.delay * RATE;
    assert!((start.expect("it is heard") as f32 - travel).abs() < 0.002 * RATE);
    assert!(
        (spot.delay - 10.14 / home.sound).abs() < 0.001,
        "{}",
        spot.delay
    );
    let side =
        |sound: &[[f32; 2]], i: usize| rms(&sound.iter().map(|frame| frame[i]).collect::<Vec<_>>());
    assert!(side(&far, 1) > 1.5 * side(&far, 0));
    let (near, away) = (side(&own, 0) + side(&own, 1), side(&far, 0) + side(&far, 1));
    assert!(away < 0.3 * near && away > 0.05 * near, "{away} {near}");
    let gone = trod_at(rock, heard(2000.0, 0.8, home), 1);
    assert!(gone.iter().all(|frame| frame == &[0.0, 0.0]));
}

/// In a vacuum the listener hears their own step through the body, dull and quieter, and no
/// step of anyone else.
#[test]
fn a_vacuum_carries_only_the_body() {
    let walk = Step::on(StepSurface::Rock);
    let air = left(&trod(walk, 2));
    let body = left(&trod_at(walk, heard(0.0, 0.0, Air::VACUUM), 2));
    assert!(rms(&body) > 0.05 * rms(&air) && rms(&body) < rms(&air));
    assert!(band(&body, 4000.0) < 0.1 * band(&air, 4000.0));
    let other = trod_at(walk, heard(3.0, 0.0, Air::VACUUM), 2);
    assert!(other.iter().all(|frame| frame == &[0.0, 0.0]));
}

/// The armour of the robot ticks on a hard landing and not on a soft one.
#[test]
fn only_a_hard_landing_ticks() {
    let land = |force: f32| {
        let step = Step {
            contact: Contact::Landing,
            force,
            mass: 25.8,
            ..Step::on(StepSurface::Soil)
        };
        let sound = left(&trod(step, 1));
        band(&sound, 3900.0) / rms(&sound)
    };
    assert!(land(1.0) > 1.4 * land(0.3), "{} {}", land(1.0), land(0.3));
}
