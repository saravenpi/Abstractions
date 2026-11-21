//! Tests of how the glow comes and goes: the plasma is a continuous function of the air and the
//! speed, the skin fades from sight within seconds once the air is gone, with no frame that
//! drops much, and no colour changes faster than a set rate.

use super::gas::incandescence;
use super::skin::glowing;
use super::tune::HUE_RATE;
use super::{Gas, Glow, entry};

/// Seconds of one frame.
const FRAME: f32 = 1.0 / 60.0;

/// Largest step of the plasma between two neighbouring airs or speeds a hundredth apart.
const STEP: f32 = 0.03;

/// Largest share of the full glow the skin or the plasma may gain or lose in one frame.
const DROP: f32 = 0.02;

/// Glow of the skin under which daylight drowns it, and under which it is lost at night.
const SEEN: (f32, f32) = (0.02, 5.0e-4);

/// Fastest change of the hue of the skin, per second.
const HUE: f32 = 1.0;

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

/// Out of the air of a typical exit the plasma is gone within three seconds; the skin is lost
/// in daylight within five seconds and at night within ten, then it is dark, never losing more
/// than a fiftieth of its full glow in a frame.
#[test]
fn the_skin_fades_from_sight_within_seconds() {
    let mut glow = Glow::NONE;
    for _ in 0..60 * 60 {
        glow = glow.follow(high(), FRAME);
    }
    assert!(glowing(glow.kelvin) > 0.9, "{}", glow.kelvin);
    let (mut frames, mut last) = (0u32, glowing(glow.kelvin));
    let (mut day, mut night) = (None, None);
    while glow.kelvin > 0.0 && frames < 600 * 60 {
        glow = glow.follow(Glow::NONE, FRAME);
        let now = glowing(glow.kelvin);
        assert!(last - now <= DROP, "frame {frames}: {last} to {now}");
        if frames == 3 * 60 {
            assert_eq!(glow.heat, 0.0);
        }
        let seconds = frames as f32 * FRAME;
        day = day.or((now < SEEN.0).then_some(seconds));
        night = night.or((now < SEEN.1).then_some(seconds));
        (last, frames) = (now, frames + 1);
    }
    let (day, night) = (day.expect("lost by day"), night.expect("lost at night"));
    assert!(day > 1.0 && day < 5.0, "lost by day after {day} s");
    assert!(night > day && night < 10.0, "lost at night after {night} s");
    assert!(frames as f32 * FRAME < 60.0);
}

/// From a cold start into the hottest entry, and from a steady entry to full speed, the hue of
/// the skin changes by at most [`HUE`] per second, and the colour of the gas by at most what
/// [`HUE_RATE`] allows.
#[test]
fn colours_change_at_a_bounded_rate() {
    let fast = entry(1.0e-4, 11_000.0, Gas::NitrogenOxygen);
    let mut glow = Glow::NONE;
    for target in [high(), fast] {
        for _ in 0..30 * 60 {
            let next = glow.follow(target, FRAME);
            if glowing(glow.kelvin) > 0.0 {
                let hue = incandescence(next.kelvin).distance(incandescence(glow.kelvin));
                assert!(hue <= HUE * FRAME, "skin hue moved {hue} in a frame");
            }
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

/// Falling into the air of an Earth at the speed of a low orbit, through the fading top of the
/// air, the plasma and the glow of the skin start from nothing and fade in: neither gains more
/// than a fiftieth of its full strength in a frame, and both end strong.
#[test]
fn the_glow_fades_in_on_the_way_in() {
    let air = |height: f32| {
        let (top, scale) = (101_000.0, 8420.0);
        let under = ((top - height) / (2.0 * scale)).clamp(0.0, 1.0);
        1.2 * (-height / scale).exp() * under * under * (3.0 - 2.0 * under)
    };
    let mut glow = Glow::NONE;
    let mut height = 102_000.0f32;
    while height > 60_000.0 {
        let next = glow.follow(
            entry(f64::from(air(height)), 7800.0, Gas::NitrogenOxygen),
            FRAME,
        );
        assert!(
            next.heat - glow.heat <= DROP,
            "the plasma jumped at {height} m"
        );
        assert!(
            next.shine - glow.shine <= DROP,
            "the skin jumped at {height} m"
        );
        (glow, height) = (next, height - 400.0 * FRAME);
    }
    assert!(
        glow.heat > 0.8 && glow.shine > 0.8,
        "{} {}",
        glow.heat,
        glow.shine
    );
}
