//! The cost of one plate sample: under 2 microseconds, the budget of the planet plan.

use std::hint::black_box;
use std::time::Instant;

use bevy::math::DVec3;

use super::tests::earth;
use crate::world::globe::tests_record::sphere_dirs;

/// Most nanoseconds one plate sample may take.
const BUDGET: f64 = 2000.0;

/// Samples per timed run.
const COUNT: u32 = 200_000;

/// One plate sample costs under 2 microseconds on scattered directions, the best of eight runs so that a
/// busy machine does not fail it.
#[test]
fn a_plate_sample_costs_under_two_microseconds() {
    let plates = earth(42);
    let dirs: Vec<DVec3> = sphere_dirs(COUNT).collect();
    let run = || {
        let start = Instant::now();
        let sum: f32 = dirs.iter().map(|&d| plates.sample(black_box(d)).land).sum();
        black_box(sum);
        start.elapsed().as_secs_f64() * 1e9 / f64::from(COUNT)
    };
    let best = (0..8).map(|_| run()).fold(f64::MAX, f64::min);
    println!("Plates::sample: {best:.0} ns");
    assert!(best < BUDGET, "a plate sample takes {best:.0} ns");
}
