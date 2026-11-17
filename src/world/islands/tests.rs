//! Tests of the gradient noise the sphere path reads.

use super::solid::gradient;

/// Returns scattered points of space, a few hundred units apart at most.
fn points() -> impl Iterator<Item = [f64; 3]> {
    (0..20_000u64).map(|i| {
        let part =
            |k: u64| ((i * 3 + k).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as f64 * 3e-5 - 250.0;
        [part(0), part(1), part(2)]
    })
}

/// The noise is zero on lattice points, continuous across cell faces and differs between seeds.
#[test]
fn gradient_noise_is_continuous() {
    assert_eq!(gradient(5, [3.0, -7.0, 12.0]), 0.0);
    let mut differ = 0;
    for p in points() {
        let wall = [p[0].floor(), p[1], p[2]];
        let (a, b) = (
            gradient(5, [wall[0] - 1e-9, p[1], p[2]]),
            gradient(5, [wall[0] + 1e-9, p[1], p[2]]),
        );
        assert!(
            (a - b).abs() < 1e-6,
            "jump of {} across the cell face at {wall:?}",
            a - b
        );
        let near = gradient(5, [p[0] + 1e-6, p[1], p[2]]);
        assert!((near - gradient(5, p)).abs() < 1e-4);
        differ += usize::from((gradient(5, p) - gradient(6, p)).abs() > 0.05);
    }
    assert!(
        differ > 15_000,
        "seeds 5 and 6 agree on {} points of 20,000",
        20_000 - differ
    );
}

/// One octave spreads like the 2D Perlin noise of the plane: within -1 to 1, deviation near 0.42, mean near 0.
#[test]
fn gradient_noise_spreads_like_the_plane() {
    let values: Vec<f64> = points().map(|p| gradient(11, p)).collect();
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let spread = (values.iter().map(|v| v * v).sum::<f64>() / n).sqrt();
    assert!(values.iter().all(|v| (-1.0..=1.0).contains(v)));
    assert!(mean.abs() < 0.02, "mean {mean}");
    assert!((0.38..0.47).contains(&spread), "deviation {spread}");
}
