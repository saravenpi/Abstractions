//! Tests of the staging of the cover maps: the budget of a frame finishes the next snapshot
//! within its period, and a map knows when a jump in time is past what staging can follow.

use super::map::{CoverMap, budget};
use crate::universe::{EARTH_RADIUS, PlanetRecord};
use crate::weather::{Field, SNAPSHOT, TEXELS};

/// Returns the empty map of a planet like home over wet ground.
fn empty() -> CoverMap {
    let field = Field::new(&PlanetRecord::TEMPERATE, 5, EARTH_RADIUS);
    CoverMap::new(field, vec![0.5; TEXELS])
}

/// At any frame rate from 4 to 144 per second, the frames of one snapshot period work out
/// twice the texels of a snapshot, so the next snapshot is done in half its period, and so are
/// two maps that share the budget of each frame.
#[test]
fn the_budget_finishes_a_snapshot_within_its_period() {
    for rate in [4.0_f32, 30.0, 60.0, 144.0] {
        let frames = SNAPSHOT as f32 * rate;
        let worked = budget(1.0 / rate) as f32 * frames;
        assert!(worked >= 2.0 * TEXELS as f32, "{rate}: {worked}");
        assert!(worked / 2.0 >= TEXELS as f32, "{rate}: two maps");
    }
}

/// The budget is a fraction of what the old pace worked out each frame at 60 frames per
/// second, and a frame of no time still moves the work on.
#[test]
fn a_frame_does_only_what_the_period_needs() {
    assert!(budget(1.0 / 60.0) < 1_200);
    assert!(budget(0.0) > 0);
}

/// A map that holds no snapshot is stale, one that holds this one and the next is not, and a
/// jump of more than one snapshot either way makes it stale.
#[test]
fn a_map_is_stale_after_a_jump_in_time() {
    let mut map = empty();
    assert!(map.stale(0.0));
    map.advance(100.0, 0);
    assert!(!map.stale(100.0));
    assert!(!map.stale(100.0 + SNAPSHOT));
    assert!(map.stale(100.0 + 3.0 * SNAPSHOT));
    assert!(map.stale(100.0 - 3.0 * SNAPSHOT));
}
