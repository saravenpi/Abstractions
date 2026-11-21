//! Steady flight sounds steady. Nothing in the voice of a ship swells at a rate a listener hears
//! as a pulse unless something real changes, and the sway of the chase camera on its spring is
//! heard neither as a pulse of level nor as a bend of pitch.

use std::f32::consts::{PI, TAU};

use super::rig::{alone, parked, rms};
use super::{RATE, frames, mixer};
use crate::audio::ship::air_tune::GUST;
use crate::audio::ship::feed::{Ch, HullSound};
use crate::audio::ship::tune::{SHAFT_HZ, SURGE, TWIN};
use crate::settings::Audio as Volumes;

/// Frames between two frames of the game.
const GAME_FRAME: usize = 800;

/// The modulations the voice keeps are out of the band where a swell reads as a pulse, from
/// 0.3 to 5 Hz: the roar of a jet swells by under half a decibel below 5 Hz, the two pumps beat
/// once in more than ten seconds, and the gusts come once in ten seconds, only in rough air.
#[test]
fn nothing_swells_in_the_band_of_a_pulse() {
    let below = SURGE.1 * (2.0 / PI * (5.0 / SURGE.0).atan()).sqrt();
    assert!(20.0 * (1.0 + below).log10() < 0.5, "{below}");
    assert!(1.0 / ((TWIN - 1.0) * SHAFT_HZ) > 10.0);
    assert!(1.0 / GUST.0 >= 10.0);
}

/// Returns the left channel of `seconds` of the ship this machine flies, heard from a chase view
/// 30 metres behind in steady cruise, the camera swaying `sway` metres on its spring at 0.7 Hz.
fn chase(sway: f32, seconds: f32) -> Vec<f32> {
    let hull = |t: f32| -> HullSound {
        parked()
            .with(Ch::Riding, 1.0)
            .with(Ch::Thrust, 0.6)
            .with(Ch::Speed, 120.0)
            .with(Ch::Height, 80.0)
            .with(Ch::Distance, 30.0 + sway * (TAU * 0.7 * t).sin())
    };
    let (mut mixer, shared) = mixer(&alone(hull(0.0)));
    (0..frames(seconds))
        .map(|i| {
            if i % GAME_FRAME == 0 {
                shared.publish(&alone(hull(i as f32 / RATE)), &Volumes::default());
            }
            mixer.frame()[0]
        })
        .collect()
}

/// In steady cruise in the chase view the camera sways 1.5 metres on its spring, and the ship
/// sounds within a decibel of the same cruise with a still camera in every 50 milliseconds: the
/// same noise, so only the sway can tell them apart.
#[test]
fn the_chase_camera_sways_without_a_pulse() {
    let (still, swaying) = (chase(0.0, 30.0), chase(1.5, 30.0));
    let window = frames(0.05);
    let skip = frames(5.0) / window;
    let worst = still
        .chunks(window)
        .zip(swaying.chunks(window))
        .skip(skip)
        .map(|(a, b)| (20.0 * (rms(b) / rms(a).max(1.0e-9)).log10()).abs())
        .fold(0.0, f64::max);
    assert!(worst < 1.0, "the sway moves the ship by {worst:.2} dB");
}
