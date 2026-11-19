//! The cost of looking for herds: bounded site samples and time.

use std::cell::Cell;
use std::time::Instant;

use bevy::math::DVec3;

use super::fixtures::{light, meadow_life, sample};
use super::{Ground, Search, near};

/// A 1 km search samples at most a few hundred sites and, hunts included, takes under 1.5 ms in an
/// optimised build: 1 ms for the herds and 0.5 ms for the hunts.
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
    let ground = Ground {
        sample: &counted,
        light: &light,
    };
    let each = (0..20)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(near(&life, &ground, &search));
            start.elapsed()
        })
        .min();
    let each = each.unwrap_or_default();
    let samples = calls.get() / 20;
    assert!(samples <= 400, "{samples} samples");
    let limit = [1_500, 20_000][usize::from(cfg!(debug_assertions))];
    assert!(each.as_micros() < limit, "{each:?}, {samples} samples");
}
