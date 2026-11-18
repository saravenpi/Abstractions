//! The cost of a home height sample: rivers, landforms and the climate keep it within 1.25 times what it
//! was before them, measured against the frozen flat sample on the same machine.

use std::hint::black_box;
use std::time::Instant;

use bevy::math::DVec3;

use crate::world::globe::Globe;

/// Globe height over flat height at the base commit for scattered points: 1629 ns against 910 ns.
const SCATTERED_BEFORE: f64 = 1629.0 / 910.0;

/// Globe height over flat height at the base commit over one 640 by 625 m patch: 1559 ns against 910 ns.
const PATCH_BEFORE: f64 = 1559.0 / 910.0;

/// Samples per timed run, the count the base commit was measured with.
const COUNT: usize = 100_000;

/// Returns the seconds of one run of `sample` over [`COUNT`] indices.
fn run(sample: &dyn Fn(usize) -> f32) -> f64 {
    let start = Instant::now();
    black_box((0..COUNT).map(|i| sample(black_box(i))).sum::<f32>());
    start.elapsed().as_secs_f64()
}

/// Returns the lowest ratio of a run of `sample` to the run of `flat` just before it, over eight rounds,
/// so that a busy machine slows both runs of a round alike.
fn ratio(flat: &dyn Fn(usize) -> f32, sample: &dyn Fn(usize) -> f32) -> f64 {
    (0..8)
        .map(|_| {
            let base = run(flat);
            run(sample) / base
        })
        .fold(f64::MAX, f64::min)
}

/// Returns [`COUNT`] directions on a golden spiral and as many on a 2 m grid around a land point of `globe`.
fn points(globe: &Globe) -> [Vec<DVec3>; 2] {
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    let spiral: Vec<DVec3> = (0..COUNT as u32)
        .map(|i| {
            let y = 1.0 - 2.0 * (f64::from(i) + 0.5) / COUNT as f64;
            let (ring, turn) = ((1.0 - y * y).sqrt(), golden * f64::from(i));
            DVec3::new(ring * turn.cos(), y, ring * turn.sin())
        })
        .collect();
    let at = spiral
        .iter()
        .copied()
        .find(|&d| globe.height(d) > 8.0)
        .expect("land");
    let (east, south) = at.any_orthonormal_pair();
    let grid = (0..COUNT as u32)
        .map(|i| {
            at * globe.radius() + (east * f64::from(i % 320) + south * f64::from(i / 320)) * 2.0
        })
        .map(DVec3::normalize)
        .collect();
    [spiral, grid]
}

/// Home height samples cost at most 1.25 times their share of a flat sample at the base commit, on
/// scattered points and over one patch the way a chunk reads them.
///
/// The base numbers were measured on the development machine, and the ratio follows the processor:
/// a hosted runner reads 2.17 where that machine reads 1.34. Run it there by hand, with
/// `cargo test --features dev home_height_costs -- --ignored`.
#[test]
#[ignore = "compares against timings of the development machine"]
fn home_height_costs_what_it_did() {
    let globe = Globe::new(42, 6000.0, 22.0);
    let [spiral, grid] = points(&globe);
    let flat = |i: usize| globe.height_flat((i % 400) as f32 * 9.7, (i / 400) as f32 * 9.7);
    let scattered = ratio(&flat, &|i| globe.height(spiral[i]));
    let patch = ratio(&flat, &|i| globe.height(grid[i]));
    println!("Globe::height over height_flat: scattered {scattered:.2}, patch {patch:.2}");
    assert!(
        scattered < SCATTERED_BEFORE * 1.25,
        "scattered {scattered:.2}"
    );
    assert!(patch < PATCH_BEFORE * 1.25, "patch {patch:.2}");
}
