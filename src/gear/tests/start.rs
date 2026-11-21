//! Glides started the ways a player starts them, on the worlds a player starts them on: from
//! the top of a jump, off a cliff at a run and out of a long fall, at home, under air thinner
//! than any home has, and on a world whose air is too thin to glide in.

use bevy::prelude::*;

use super::pilot::keys;
use super::{DT, Flight};
use crate::gear::tune::{MOST_SINK, REF_GRAVITY};
use crate::gear::{GearInput, Mode, glides};
use crate::universe::PlanetRecord;
use crate::wind::thickness;

/// Fastest forward speed in metres per second of a flyable glide: the 50 m/s of a dive.
const FORWARD: f32 = 50.0;

/// Lowest glide ratio of a flyable glide, the low end of the tuned glide of four to one.
const RATIO: f32 = 3.5;

/// Seconds after the wings are asked open by which the glide is flyable, and the seconds it is
/// then watched for, one second at a time. A glide at the edge of the rule of a home flies 1.4
/// times as fast and settles that much slower than at home.
const SETTLE: (f32, u32) = (12.0, 15);

/// The worlds of the test: a name, the gravity in metres per second squared and the thickness
/// of the air against home. Home air under standard gravity; and two edges thinner than any
/// home, half the home air per standard gravity of pull, under 1 g and under 1.3 g. The thinnest
/// air of a home is 0.65 of the home air under at most 1.05 g, see `universe::PRESSURE`.
const WORLDS: [(&str, f32, f32); 3] = [
    ("home", REF_GRAVITY, 1.0),
    ("edge", REF_GRAVITY, 0.5),
    ("heavy edge", 1.3 * REF_GRAVITY, 0.65),
];

/// The starts of the test: a name and the velocity over the ground at the press, in metres per
/// second: the top of a jump at a walk, a run off a cliff, and a fall of two and a half seconds.
const STARTS: [(&str, Vec3); 3] = [
    ("jump", Vec3::new(0.0, 0.3, -5.0)),
    ("cliff", Vec3::new(0.0, -1.0, -8.5)),
    ("fall", Vec3::new(0.0, -25.0, -1.0)),
];

/// Returns how thick the air of a temperate world under `pressure` bar is against home.
fn air(pressure: f64) -> f32 {
    thickness(&PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    }) as f32
}

/// Returns a body falling at `velocity` with no gear in use, under `gravity` in air `air`
/// thick, for a third of a second as the controller hands it to the gear, looking ahead a
/// little down: the attitude it has when the wings are asked open.
fn falling(velocity: Vec3, (gravity, air): (f32, f32)) -> Flight {
    let mut flight = Flight::new(Mode::Off, velocity);
    let folded = GearInput {
        gravity,
        air,
        aim: Some(Vec2::new(0.0, -0.2)),
        ..GearInput::default()
    };
    flight.run(&folded, 1.0 / 3.0);
    flight
}

/// Returns the mean forward speed and sink rate, in metres per second, over each second of
/// a glide opened from `velocity` and flown hands off, once it settled, see [`SETTLE`].
fn opened(velocity: Vec3, world: (f32, f32)) -> Vec<(f32, f32)> {
    let mut flight = falling(velocity, world);
    let input = GearInput {
        gravity: world.0,
        air: world.1,
        ..keys(0.0, 0.0)
    };
    flight.run(&input, SETTLE.0);
    let frames = (1.0 / DT).round() as u32;
    (0..SETTLE.1)
        .map(|_| {
            let (mut forward, mut sink) = (0.0, 0.0);
            for _ in 0..frames {
                flight.tick(&input, DT);
                forward += flight.state.velocity.with_y(0.0).length() / frames as f32;
                sink -= flight.state.velocity.y / frames as f32;
            }
            (forward, sink)
        })
        .collect()
}

/// At home and at the edge of the rule of a home, a glide opened from a jump, off a cliff and
/// out of a fall is flyable within twelve seconds and stays so: every second after it sinks under
/// 11 m/s, flies forward under 50 m/s and glides at least 3.5 to one. The air carries it there,
/// see `gear::glides`.
#[test]
fn every_start_reaches_a_flyable_glide_and_keeps_it() {
    for (world, gravity, thick) in WORLDS {
        assert!(
            glides(thick, 1.2, gravity),
            "{world}: the air does not carry"
        );
        for (start, velocity) in STARTS {
            let seconds = opened(velocity, (gravity, thick));
            println!("{world} {start}: {seconds:?}");
            for (second, (forward, sink)) in seconds.into_iter().enumerate() {
                let what =
                    format!("{world} {start} second {second}: {forward} forward, {sink} down");
                assert!(sink > 0.0 && sink <= MOST_SINK, "{what}");
                assert!(forward <= FORWARD && forward / sink >= RATIO, "{what}");
            }
        }
    }
}

/// On a world of 0.02 bar the air does not carry the wings, which the interface then says,
/// and the glide falls: twelve seconds after the wings opened from every start it still sinks
/// faster than any glide, under the gravity of home.
#[test]
fn a_thin_world_cannot_glide() {
    let thin = air(0.02);
    assert!(thin > 0.0 && !glides(thin, 1.2, REF_GRAVITY), "air {thin}");
    assert!(glides(air(1.0), 1.2, REF_GRAVITY));
    for (start, velocity) in STARTS {
        let seconds = opened(velocity, (REF_GRAVITY, thin));
        let slowest = seconds
            .iter()
            .map(|(_, sink)| *sink)
            .fold(f32::MAX, f32::min);
        assert!(
            slowest > MOST_SINK,
            "{start}: sinks at {slowest} m/s in thin air"
        );
    }
}
