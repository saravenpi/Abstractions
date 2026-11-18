//! Wingsuit alone: a real glide that always loses energy, steered by the keys.

use bevy::prelude::*;

use super::measure::{dive, settled_glide};
use super::{DT, Flight, gliding, looking};
use crate::gear::{GearInput, Mode};

/// Returns an input holding the wings open with the pitch and roll sticks at `pitch` and
/// `roll`, the view straight ahead.
pub(super) fn sticks(pitch: f32, roll: f32) -> GearInput {
    GearInput {
        pitch,
        roll,
        ..gliding(0.0)
    }
}

/// The range of the hands-off glide ratio under the home gravity and air.
const GLIDE: (f32, f32) = (3.5, 4.5);

/// The range of the hands-off sink rate under the home gravity and air, in metres per second.
const SINK: (f32, f32) = (6.0, 8.0);

/// Speed in metres per second a dive must reach.
const DIVE: f32 = 50.0;

/// Let go, the wings settle on a glide of about four to one that sinks at about 7 m/s, at about
/// 30 m/s, wherever the view looks.
#[test]
fn hands_off_glide_is_about_four_to_one() {
    let (ratio, speed, sink) = settled_glide(&gliding(0.0));
    assert!((GLIDE.0..=GLIDE.1).contains(&ratio), "ratio {ratio}");
    assert!((SINK.0..=SINK.1).contains(&sink), "sink {sink}");
    assert!((25.0..=36.0).contains(&speed), "speed {speed}");
    for look in [looking(1.2, 0.8), looking(-2.0, -1.0)] {
        let looked = settled_glide(&GearInput {
            look,
            ..gliding(0.0)
        });
        assert_eq!(looked, (ratio, speed, sink), "the view steered");
    }
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

/// The forward key held a second and let go dives past 50 m/s, and the body pulls out by itself.
#[test]
fn a_dive_trades_height_for_speed() {
    let mut flight = dive(1.2);
    assert!(
        flight.state.airspeed > DIVE,
        "dive {}",
        flight.state.airspeed
    );
    flight.run(&gliding(0.0), 30.0);
    let (ratio, _, sink) = settled_glide(&gliding(0.0));
    let v = flight.state.velocity;
    assert!((v.with_y(0.0).length() / -v.y - ratio).abs() < 0.3, "{v}");
    assert!((-v.y - sink).abs() < 1.0, "{v}");
}

/// The back key held 2.2 s out of a dive pulls out and climbs 15 m and more above the bottom
/// of the pull-out, then the wing stalls under 18 m/s, drops its nose and gathers speed again.
/// The climb still costs energy.
#[test]
fn pull_up_climbs_then_stalls_and_recovers() {
    let mut flight = dive(1.2);
    let before = flight.energy(22.0);
    let mut track = [(0.0_f64, 0.0_f32); 600];
    for (frame, at) in track.iter_mut().enumerate() {
        let stick = f32::from(u8::from(frame < 132));
        flight.tick(&sticks(stick, 0.0), DT);
        *at = (flight.pos.y, flight.state.airspeed);
    }
    let low = (0..120).min_by(|a, b| track[*a].0.total_cmp(&track[*b].0));
    let bottom = track[low.unwrap_or(0)].0;
    let top = track
        .iter()
        .map(|at| at.0)
        .skip(low.unwrap_or(0))
        .fold(f64::MIN, f64::max);
    let slowest = track.iter().map(|at| at.1).fold(f32::MAX, f32::min);
    assert!(top > bottom + 15.0, "climbed only {}", top - bottom);
    assert!(slowest < 18.0, "never stalled: {slowest}");
    assert!(flight.state.airspeed > slowest + 8.0, "no speed back");
    assert!(flight.energy(22.0) < before, "the pull-up gained energy");
    assert!(
        flight.pos.y < top - 20.0 && flight.twist < 0.09,
        "the climb did not end"
    );
}

/// No key held steadily holds height without the boosters. The slowest sink, about 5 m/s,
/// comes with the back key held a little: the wing flies slow near its stall angle.
#[test]
fn glide_cannot_hold_height() {
    for stick in [-1.0, -0.3, 0.0, 0.3, 1.0] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -40.0));
        flight.run(&sticks(stick, 0.0), 60.0);
        assert!(
            flight.pos.y < -240.0,
            "stick {stick}: lost only {}",
            -flight.pos.y
        );
    }
}
