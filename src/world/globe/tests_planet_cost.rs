//! The cost of the ground of a planet of real size against a planet the local ground shapes alone: the
//! plates, the shelf and the regional band may add at most one and a half times a local sample.

use std::hint::black_box;
use std::time::Instant;

use bevy::math::DVec3;

use super::tests_record::sphere_dirs;
use super::{Globe, Grain};

/// Most a ground sample of a large planet may cost, in times the sample of a small one.
const TIMES: f64 = 2.5;

/// Samples per timed run.
const COUNT: u32 = 100_000;

/// Returns the nanoseconds one ground sample of `globe` takes over `dirs`, as a reader too coarse for
/// rivers takes it, so that no atlas cell is built.
fn cost(globe: &Globe, dirs: &[DVec3]) -> f64 {
    let start = Instant::now();
    let sum: f32 = dirs
        .iter()
        .map(|&d| globe.height_seen(black_box(d), Grain::Far))
        .sum();
    black_box(sum);
    start.elapsed().as_secs_f64() * 1e9 / dirs.len() as f64
}

/// A ground sample of a planet 6,000 km in radius costs at most 2.5 times the sample of a 6 km one: the
/// lowest ratio of eight rounds, each timing both planets one after the other, once the shore of the
/// large one is measured.
#[test]
fn a_large_planet_costs_little_more_than_a_small_one() {
    let (small, large) = (Globe::new(42, 6000.0, 22.0), Globe::new(42, 6.0e6, 9.81));
    let dirs: Vec<DVec3> = sphere_dirs(COUNT).collect();
    black_box(large.height_seen(DVec3::X, Grain::Far));
    let rounds = (0..8).map(|_| {
        let base = cost(&small, &dirs);
        (cost(&large, &dirs) / base, base)
    });
    let (ratio, base) = rounds.fold((f64::MAX, 0.0), |best, round| match round.0 < best.0 {
        true => round,
        false => best,
    });
    println!("ground sample: {base:.0} ns on 6 km, {ratio:.2} times that on 6,000 km");
    assert!(
        ratio < TIMES,
        "a large planet costs {ratio:.2} times a small one"
    );
}
