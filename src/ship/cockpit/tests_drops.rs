//! Tests of the water on the canopy: the glass wets in cloud and clears a few seconds after it at
//! speed, the drops bead at rest and stream back toward the tail at speed, and every drop lies on
//! the glass, never in the cabin or in the air ahead of it.

use bevy::prelude::*;

use super::wet::{DROPS, LIFT, TRAVEL, placed, seeds, stretch, way, wetted};

/// Seconds of one frame.
const FRAME: f32 = 1.0 / 60.0;

/// Returns the wetness after `seconds` of flight at `speed` in air as wet as `soak`, from `wet`.
fn after(mut wet: f32, soak: f32, speed: f32, seconds: f32) -> f32 {
    for _ in 0..(seconds / FRAME) as u32 {
        wet = wetted(wet, soak, speed, FRAME);
    }
    wet
}

/// In a cloud the glass is wet within a few seconds; out of it at speed it clears within eight
/// seconds, and at rest it stays wet far longer.
#[test]
fn the_glass_wets_in_cloud_and_clears_after() {
    let soaked = after(0.0, 1.0, 120.0, 4.0);
    assert!(soaked > 0.9, "{soaked}");
    let flown = after(soaked, 0.0, 150.0, 8.0);
    assert!(flown < 0.05, "still {flown} after eight seconds at speed");
    assert_eq!(after(soaked, 0.0, 150.0, 30.0), 0.0);
    assert!(after(soaked, 0.0, 0.0, 8.0) > 0.4);
}

/// At rest a drop creeps down its pane as a round bead; at speed it runs back toward the tail
/// over its pane, drawn out into a streak.
#[test]
fn drops_bead_at_rest_and_stream_at_speed() {
    let pane = Vec3::new(0.0, 0.6, -0.8).normalize();
    let still = way(pane, 0.0);
    let fast = way(pane, 150.0);
    assert!(still.y < -0.5 && still.dot(pane).abs() < 1.0e-5, "{still}");
    assert!(fast.z > 0.5 && fast.dot(pane).abs() < 1.0e-5, "{fast}");
    assert_eq!(stretch(0.0), 1.0);
    assert!(stretch(150.0) > 3.0);
}

/// Every drop of the canopy sits a few millimetres out from its pane of glass, within the run of
/// its pane, whatever the airspeed and the moment.
#[test]
fn every_drop_lies_on_the_glass() {
    let all = seeds();
    assert_eq!(all.len(), DROPS);
    for seed in &all {
        for (seconds, speed) in [(0.0, 0.0), (3.7, 40.0), (11.2, 180.0)] {
            let (middle, along) = placed(seed, seconds, speed);
            let off = (middle - seed.at).dot(seed.normal);
            assert!((off - LIFT).abs() < 1.0e-4, "{off} off the glass");
            assert!(middle.distance(seed.at) <= 0.5 * TRAVEL + LIFT + 1.0e-4);
            assert!(along.dot(seed.normal).abs() < 1.0e-4);
        }
    }
}
