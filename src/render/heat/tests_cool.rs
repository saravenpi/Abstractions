//! Tests of how the glow comes and goes: the plasma is a continuous function of the air and the
//! speed, the skin cools by radiation over tens of seconds once the air is gone, with no frame
//! that drops much, and no colour changes faster than a set rate.

use bevy::prelude::*;

use super::gas::incandescence;
use super::skin::glowing;
use super::tune::HUE_RATE;
use super::{Gas, Glow, entry};

/// Seconds of one frame.
const FRAME: f32 = 1.0 / 60.0;

/// Largest step of the plasma between two neighbouring airs or speeds a hundredth apart.
const STEP: f32 = 0.03;

/// Largest share of its full glow the skin may lose in one frame.
const DROP: f32 = 0.01;

/// Fastest change of the seen colour of the skin, per second.
const SKIN_RATE: f32 = 2.0;

/// Returns the colour of the skin as it is seen: its colour times how strongly it glows.
fn seen(glow: &Glow) -> Vec3 {
    incandescence(glow.kelvin) * glowing(glow.kelvin)
}

/// The glow of an entry from low orbit, high in the air.
fn high() -> Glow {
    entry(1.0e-4, 7800.0, Gas::NitrogenOxygen)
}

/// From no air to thick air and from slow to fast the plasma rises from 0 to full with no step
/// between neighbours a hundredth apart: it reaches 0 continuously, never through a threshold.
#[test]
fn the_plasma_has_no_step_in_density_or_speed() {
    let at = |density: f64, speed: f64| entry(density, speed, Gas::NitrogenOxygen).heat;
    let thin = (0..=2000).map(|k| at(1.0e-9 * 1.01f64.powi(k), 7800.0));
    let slow = (0..=500).map(|k| at(1.0e-4, 100.0 * 1.01f64.powi(k)));
    for line in [thin.collect::<Vec<_>>(), slow.collect::<Vec<_>>()] {
        assert_eq!(line[0], 0.0);
        assert_eq!(*line.last().expect("samples"), 1.0);
        for pair in line.windows(2) {
            assert!((pair[1] - pair[0]).abs() < STEP, "{pair:?}");
        }
    }
}

/// Out of the air the plasma is gone within three seconds while the skin still glows, and the
/// skin then cools to dark over tens of seconds, never losing more than a hundredth of its full
/// glow in a frame.
#[test]
fn the_skin_cools_slowly_after_the_air() {
    let mut glow = Glow::NONE;
    for _ in 0..60 * 60 {
        glow = glow.follow(high(), FRAME);
    }
    assert!(glowing(glow.kelvin) > 0.9, "{}", glow.kelvin);
    let mut frames = 0u32;
    let mut last = glowing(glow.kelvin);
    while glow.kelvin > 0.0 && frames < 600 * 60 {
        glow = glow.follow(Glow::NONE, FRAME);
        let now = glowing(glow.kelvin);
        assert!(last - now <= DROP, "frame {frames}: {last} to {now}");
        if frames == 3 * 60 {
            assert_eq!(glow.heat, 0.0);
            assert!(now > 0.5, "the skin is dark after three seconds: {now}");
        }
        (last, frames) = (now, frames + 1);
    }
    let seconds = frames as f32 * FRAME;
    assert!((10.0..300.0).contains(&seconds), "dark after {seconds} s");
}

/// From a cold start into the hottest entry, and from a steady entry to full speed, the seen
/// colour of the skin changes by at most [`SKIN_RATE`] per second, and the colour of the gas by
/// at most what [`HUE_RATE`] allows.
#[test]
fn colours_change_at_a_bounded_rate() {
    let fast = entry(1.0e-4, 11_000.0, Gas::NitrogenOxygen);
    let mut glow = Glow::NONE;
    for target in [high(), fast] {
        for _ in 0..30 * 60 {
            let next = glow.follow(target, FRAME);
            let skin = seen(&next).distance(seen(&glow));
            assert!(
                skin <= SKIN_RATE * FRAME,
                "skin colour moved {skin} in a frame"
            );
            if glow.heat > 0.0 {
                let gas = next.color.distance(glow.color);
                assert!(
                    gas <= HUE_RATE * FRAME * 2.0,
                    "gas colour moved {gas} in a frame"
                );
            }
            glow = next;
        }
    }
}
