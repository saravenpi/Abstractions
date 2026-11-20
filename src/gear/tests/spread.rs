//! The wing spread: one eased motion that the eye and the lift share, the same at any frame rate.

use bevy::prelude::*;

use super::{Flight, gliding};
use crate::gear::tune::{REF_GRAVITY, WING_TIME};
use crate::gear::{GearInput, Mode};

/// Returns the spread after each frame of a fall at 30 m/s whose wings are asked open for
/// half the time and folded for the rest, at `rate` frames a second for 1.6 s, and the largest
/// change of velocity between two frames, in metres per second squared.
fn opening(rate: u32) -> (Vec<f32>, f32) {
    let dt = 1.0 / rate as f32;
    let mut flight = Flight::new(Mode::Fall, Vec3::new(0.0, -30.0, -4.0));
    let frames = rate * 8 / 5;
    let mut spreads = vec![0.0];
    for frame in 0..frames {
        let input = match frame < frames / 2 {
            true => gliding(-0.6),
            false => GearInput::default(),
        };
        flight.tick(&input, dt);
        spreads.push(flight.state.spread());
    }
    (spreads, flight.kick / dt)
}

/// Returns the largest change of `values` between two frames and the largest change of that
/// change, each per second and per second squared at `rate` frames a second.
fn steepest(values: &[f32], rate: u32) -> (f32, f32) {
    let rate = rate as f32;
    let speeds: Vec<f32> = values.windows(2).map(|w| (w[1] - w[0]) * rate).collect();
    let bends = speeds.windows(2).map(|w| (w[1] - w[0]).abs() * rate);
    (
        speeds
            .iter()
            .fold(0.0_f32, |most, speed| most.max(speed.abs())),
        bends.fold(0.0, f32::max),
    )
}

/// The spread is one eased motion of 0.35 to 0.5 s each way: it starts and ends at rest, never
/// faster than its ease allows, and its rate never changes faster than the ease bends, at 30
/// and at 144 frames a second. A step at either end would show as a rate or a bend far above.
#[test]
fn the_spread_and_its_rate_are_continuous() {
    assert!(
        (0.35..=0.5).contains(&WING_TIME),
        "wings take {WING_TIME} s"
    );
    let (fastest, sharpest) = (1.5 / WING_TIME, 6.0 / (WING_TIME * WING_TIME));
    for rate in [30, 144] {
        let (spreads, _) = opening(rate);
        let (speed, bend) = steepest(&spreads, rate);
        assert!(speed <= fastest * 1.01, "{rate} Hz: {speed} per second");
        assert!(bend <= sharpest * 1.05, "{rate} Hz: bends at {bend}");
        let open = spreads[(rate * 4 / 5) as usize];
        let shut = spreads[spreads.len() - 1];
        assert_eq!((open, shut), (1.0, 0.0), "{rate} Hz: not finished");
        let half = spreads[(WING_TIME * 0.5 * rate as f32).round() as usize];
        assert!((half - 0.5).abs() < 0.12, "{rate} Hz: {half} at half time");
    }
}

/// The same motion at both frame rates: the spread a fifth of a second after the wings are
/// asked open does not depend on how the time was cut.
#[test]
fn the_spread_is_the_same_at_any_frame_rate() {
    let at = |rate: u32| opening(rate).0[(rate / 5) as usize];
    let (slow, fast) = (at(30), at(120));
    assert!((slow - fast).abs() < 1e-3, "{slow} against {fast}");
}

/// The lift follows the spread the eye sees: while the wings open and fold in a fall the
/// velocity never changes by more than 3 g from one frame to the next, at either frame rate.
#[test]
fn opening_the_wings_does_not_jerk_the_body() {
    for rate in [30, 144] {
        let (_, kick) = opening(rate);
        assert!(kick < 3.0 * REF_GRAVITY, "{rate} Hz: a kick of {kick} m/s2");
    }
}
