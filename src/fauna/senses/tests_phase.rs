//! Tests of the phased senses: an animal that senses every third frame with three times the step
//! ends up as aware as one that senses every frame.

use bevy::math::{DVec3, Vec2, Vec3};

use super::feel::{Air, Senser, Threat, perceive};
use super::meter::Meter;
use super::tests::{grazer, robot};
use super::tune::SENSE_EVERY;

/// Frames of the run, 3 s at 60 frames per second.
const FRAMES: u32 = 180;

/// Returns the awareness of `me` of `threat` after 3 s when it senses every `every` frames.
fn aware_after(me: &Senser, threat: Threat, every: u32) -> f32 {
    let flat = |_: Vec2| 0.0;
    let air = Air {
        wind: DVec3::ZERO,
        ground: &flat,
        dt: every as f32 / 60.0,
    };
    let mut meter = Meter::default();
    for _ in 0..FRAMES / every {
        perceive(&mut meter, me, &[threat], &air);
    }
    meter.of(threat.key)
}

/// A still robot 45 m away fills the meter of a grazer that senses every third frame to within 2
/// percent of the meter of one that senses every frame, after 3 s.
#[test]
fn a_third_of_the_senses_fill_the_meter_alike() {
    let me = grazer(1.0);
    let still = robot(&me, Vec3::new(45.0, 0.0, 0.0), (0.0, 0.0));
    let every = aware_after(&me, still, 1);
    let third = aware_after(&me, still, SENSE_EVERY);
    assert!(every > 0.3 && every < 1.0, "{every}");
    assert!(
        (third - every).abs() <= 0.02 * every,
        "{third} against {every}"
    );
}
