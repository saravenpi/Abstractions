//! Tests of the height of the waves under the camera.

use bevy::prelude::*;

use super::swell::crest;
use super::waves::waves_for;
use crate::wind::Wind;

/// The waves under the camera stay within their summed amplitude, rise and fall in open
/// water and lie flat on the shore.
#[test]
fn crest_follows_the_waves_and_calms_ashore() {
    let wind = Wind {
        direction: Vec2::X,
        strength: 1.0,
        air: 1.0,
    };
    let waves = waves_for(&wind);
    let top: f32 = waves.iter().map(|w| w.amplitude).sum();
    let open: Vec<f32> = (0..200)
        .map(|i| {
            crest(
                &waves,
                (Vec2::new(i as f32 * 0.7, 3.0), 0.0),
                i as f32 * 0.1,
                10.0,
            )
        })
        .collect();
    assert!(open.iter().all(|h| h.abs() <= top + 1.0e-6));
    assert!(open.iter().any(|h| *h > 0.05) && open.iter().any(|h| *h < -0.05));
    assert_eq!(crest(&waves, (Vec2::new(4.0, 1.0), 0.0), 2.0, 0.0), 0.0);
}
