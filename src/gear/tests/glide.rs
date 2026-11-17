//! Wingsuit alone: a real glide that always loses energy.

use bevy::prelude::*;

use super::measure::settled_glide;
use super::{DT, Flight, gliding, looking};
use crate::gear::{GearInput, Mode};

/// The documented range of the best glide ratio.
const BEST_GLIDE: (f32, f32) = (2.5, 3.0);

/// Returns the best glide ratio over every look angle from 40 degrees down to 10 up.
fn best_glide() -> f32 {
    (-40..=10)
        .map(|degrees| settled_glide((degrees as f32).to_radians()).0)
        .fold(0.0, f32::max)
}

/// The best glide ratio is inside the documented range.
#[test]
fn best_glide_ratio_is_in_range() {
    let best = best_glide();
    assert!((BEST_GLIDE.0..=BEST_GLIDE.1).contains(&best), "best {best}");
}

/// Returns the look and sticks of a restless pilot at `time`: dives, pull-ups and turns.
fn restless(time: f32) -> GearInput {
    GearInput {
        pitch: (time * 1.3).sin(),
        roll: (time * 0.9).cos(),
        look: looking(time * 0.3, 0.9 * (time * 0.5).sin() - 0.3),
        ..gliding(0.0)
    }
}

/// Whatever the pilot does, a glide never gains mechanical energy from one frame to the next.
#[test]
fn glide_never_gains_energy() {
    for gravity in [6.6, 22.0, 39.6] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -5.0, -45.0));
        let mut energy = flight.energy(gravity);
        for frame in 0..3600 {
            let input = GearInput {
                gravity,
                ..restless(frame as f32 * DT)
            };
            flight.tick(&input, DT);
            let next = flight.energy(gravity);
            assert!(next <= energy + 1e-3, "frame {frame}: {energy} -> {next}");
            energy = next;
        }
    }
}

/// Pulling up out of a dive climbs for a while, and the climb still costs energy.
#[test]
fn pull_up_climbs_but_ends_lower_in_energy() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(-0.8), 20.0);
    let before = flight.energy(22.0);
    let mut heights = [0.0_f64; 600];
    for height in &mut heights {
        flight.tick(&gliding(0.7), DT);
        *height = flight.pos.y;
    }
    let bottom = heights[..300].iter().copied().fold(f64::MAX, f64::min);
    let top = heights[150..450].iter().copied().fold(f64::MIN, f64::max);
    assert!(top > bottom + 15.0, "climbed only {}", top - bottom);
    assert!(flight.energy(22.0) < before, "the pull-up gained energy");
    assert!(flight.pos.y < top - 20.0, "the climb did not end");
}

/// Diving gains speed, and no look angle holds height without the boosters.
#[test]
fn glide_cannot_hold_height() {
    let (_, dive) = settled_glide(-0.8);
    let (_, cruise) = settled_glide(-0.35);
    assert!(dive > cruise + 15.0, "dive {dive} cruise {cruise}");
    for degrees in [-10, 0, 10, 30, 60] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -40.0));
        flight.run(&gliding((degrees as f32).to_radians()), 60.0);
        assert!(
            flight.pos.y < -300.0,
            "look {degrees}: lost only {}",
            -flight.pos.y
        );
    }
}

/// Looking to one side banks the wings that way and turns the flight path to the look
/// direction, without swinging past it.
#[test]
fn bank_turns_the_flight_path() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    let input = GearInput {
        look: looking(-1.2, -0.35),
        ..gliding(0.0)
    };
    flight.run(&input, 0.5);
    assert!(flight.state.bank > 0.2, "looking right must bank right");
    let mut widest = 0.0_f32;
    for _ in 0..900 {
        flight.tick(&input, DT);
        widest = widest.max(flight.state.velocity.with_y(0.0).normalize().x);
    }
    let heading = flight.state.velocity.with_y(0.0).normalize();
    assert!(heading.dot(looking(-1.2, 0.0)) > 0.995, "heading {heading}");
    assert!(
        flight.state.bank.abs() < 0.05,
        "wings not level: {}",
        flight.state.bank
    );
    assert!(
        widest <= looking(-1.2, 0.0).x + 0.03,
        "swung past the look direction"
    );
}
