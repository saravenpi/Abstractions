//! The cost of looking for herds: bounded site samples and time.

use std::cell::Cell;
use std::time::Instant;

use bevy::math::DVec3;

use super::fixtures::{meadow_life, sample};
use super::{Search, near};

/// A 1 km search samples at most a few hundred sites and takes under 1 ms in an optimised build.
#[test]
fn near_is_cheap() {
    let life = meadow_life();
    let calls = Cell::new(0u32);
    let counted = |dir: DVec3| {
        calls.set(calls.get() + 1);
        sample(dir, 1.0)
    };
    let at = DVec3::new(1.0, 0.3, 0.2).normalize();
    let search = Search {
        at,
        radius: 1000.0,
        planet: 6000.0,
        time: 4321.0,
        pin: Some(at),
    };
    let start = Instant::now();
    let rounds = 20;
    for _ in 0..rounds {
        std::hint::black_box(near(&life, &counted, &search));
    }
    let each = start.elapsed() / rounds;
    let samples = calls.get() / rounds;
    assert!(samples <= 400, "{samples} samples");
    let limit = [1_000, 15_000][usize::from(cfg!(debug_assertions))];
    assert!(each.as_micros() < limit, "{each:?}, {samples} samples");
}
