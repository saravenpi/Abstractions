//! Wingsuit alone: a real glide that always loses energy, flown by the keys or by the view.

use bevy::prelude::*;

use super::measure::{dive, settled_glide};
use super::pilot::keys;
use super::{DT, Flight, gliding};
use crate::gear::tune::DIVE_LIMIT;
use crate::gear::{GearInput, Mode};

/// The range of the hands-off glide ratio under the home gravity and air.
const GLIDE: (f32, f32) = (3.5, 4.5);

/// The range of the hands-off sink rate under the home gravity and air, in metres per second.
const SINK: (f32, f32) = (5.5, 8.0);

/// Speed in metres per second a dive must reach.
const DIVE: f32 = 50.0;

/// Hands off, the wings trim to the best glide: about four to one, sinking at about 7 m/s, at
/// about 30 m/s. A view held along a slope of four to one flies the same glide.
#[test]
fn hands_off_glide_is_about_four_to_one() {
    for input in [keys(0.0, 0.0), gliding(-0.25_f32.atan())] {
        let (ratio, speed, sink) = settled_glide(&input);
        assert!((GLIDE.0..=GLIDE.1).contains(&ratio), "ratio {ratio}");
        assert!((SINK.0..=SINK.1).contains(&sink), "sink {sink}");
        assert!((25.0..=36.0).contains(&speed), "speed {speed}");
    }
}

/// A view held level or above, which the wings cannot hold, does not stall them: the stall
/// protection lowers the nose and they fly a glide no slower than the stall speed.
#[test]
fn an_aim_too_high_flies_the_best_glide() {
    for elevation in [0.0, 0.4] {
        let (ratio, speed, _) = settled_glide(&gliding(elevation));
        assert!((GLIDE.0 - 0.5..=GLIDE.1).contains(&ratio), "ratio {ratio}");
        assert!(speed > 14.0, "speed {speed}");
    }
}

/// With the aim held 20 degrees below the horizon the flight path settles within 3 degrees of
/// it, faster than the glide of four to one: the body flies where it is aimed.
#[test]
fn the_path_settles_on_an_aim_below_the_horizon() {
    let aim = -20.0_f32.to_radians();
    let (_, slow, _) = settled_glide(&gliding(-0.25_f32.atan()));
    let (ratio, speed, _) = settled_glide(&gliding(aim));
    let path = -(1.0 / ratio).atan();
    assert!(
        (path - aim).abs() < 3.0_f32.to_radians(),
        "path {path} for an aim at {aim}"
    );
    assert!(speed > slow + 2.0, "{speed} m/s against {slow}");
}

/// Returns the view and keys of a restless pilot at `time`: dives, pull-ups and turns, flat
/// and banked, and the view steering every other few seconds.
fn restless(time: f32) -> GearInput {
    let view = Vec2::new(time * 0.3, 0.9 * (time * 0.5).sin() - 0.3);
    GearInput {
        pitch: (time * 1.3).sin(),
        roll: (time * 0.9).cos(),
        yaw: (time * 0.7).sin(),
        aim: ((time * 0.4).sin() > 0.0).then_some(view),
        ..keys(0.0, 0.0)
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

/// The forward key held dives past 50 m/s, the path never steeper than the dive limit. Let go,
/// the nose comes back up to the glide by itself.
#[test]
fn a_dive_trades_height_for_speed() {
    let mut flight = dive(4.0);
    let fast = flight.state.airspeed;
    assert!(fast > DIVE, "dive {fast}");
    let path = flight
        .state
        .velocity
        .y
        .atan2(flight.state.velocity.with_y(0.0).length());
    assert!(path > DIVE_LIMIT - 0.05 && path < -0.9, "path {path}");
    flight.run(&keys(0.0, 0.0), 6.0);
    let v = flight.state.velocity;
    let path = v.y.atan2(v.with_y(0.0).length());
    assert!(path > -0.4, "the dive did not level out: {path}");
}

/// The back key held out of a fast dive pulls out and climbs 15 m and more above the bottom
/// of the pull-out while the speed bleeds away. The stall protection then drops the nose with
/// the key still held, and let go the glide speed comes back. The climb still costs energy.
#[test]
fn pull_up_climbs_then_stalls_and_recovers() {
    let mut flight = dive(2.0);
    let (before, fast) = (flight.energy(22.0), flight.state.airspeed);
    let mut track = [(0.0_f64, 0.0_f32); 900];
    for (frame, at) in track.iter_mut().enumerate() {
        let stick = f32::from(u8::from(frame < 300));
        flight.tick(&keys(stick, 0.0), DT);
        *at = (flight.pos.y, flight.state.airspeed);
    }
    let low = (0..300).min_by(|a, b| track[*a].0.total_cmp(&track[*b].0));
    let bottom = track[low.unwrap_or(0)].0;
    let top = track
        .iter()
        .map(|at| at.0)
        .skip(low.unwrap_or(0))
        .fold(f64::MIN, f64::max);
    let slowest = track.iter().map(|at| at.1).fold(f32::MAX, f32::min);
    assert!(top > bottom + 15.0, "climbed only {}", top - bottom);
    assert!(slowest > 12.0 && slowest < fast - 30.0, "kept {slowest}");
    assert!(flight.state.airspeed > slowest + 5.0, "no speed back");
    assert!(flight.energy(22.0) < before, "the pull-up gained energy");
    assert!(flight.twist < 0.09, "tumbled by {}", flight.twist);
}

/// No key held steadily holds height without the boosters, and no aim does.
#[test]
fn glide_cannot_hold_height() {
    for stick in [-1.0, -0.3, 0.0, 0.3, 1.0] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -40.0));
        flight.run(&keys(stick, 0.0), 60.0);
        let lost = -flight.pos.y;
        assert!(lost > 240.0, "stick {stick}: lost only {lost}");
    }
    for elevation in [0.0, 0.3, 0.9, 1.4] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -40.0));
        flight.run(&gliding(elevation), 60.0);
        let lost = -flight.pos.y;
        assert!(lost > 240.0, "aim {elevation}: lost only {lost}");
    }
}
