//! Tests of the relief the physics gives the sphere: mountains by gravity, continents by size, the legacy
//! plane untouched and the home ground as it was.

use super::Globe;
use super::scale::scale;
use super::tests_record::sphere_dirs;
use crate::universe::{EARTH_RADIUS, STANDARD_GRAVITY};
use crate::world::SEA_LEVEL;

/// The home planet gets factors of exactly 1; the mountain factor is the square root of the home gravity
/// over the gravity within 0.6 to 1.6, the continent factor the home radius over the radius.
#[test]
fn relief_factors_follow_gravity_and_radius() {
    assert_eq!(scale(EARTH_RADIUS, STANDARD_GRAVITY), (1.0, 1.0));
    assert!((scale(6000.0, 22.0 / 1.21).0 - 1.1).abs() < 1e-6);
    assert_eq!(scale(6000.0, 22.0 / 4.0).0, 1.6);
    assert_eq!(scale(6000.0, 22.0 * 4.0).0, 0.6);
    assert_eq!(scale(12_000.0, 22.0).1, 0.5);
    assert_eq!(scale(3000.0, 22.0).1, 2.0);
}

/// Under a weaker pull the summits stand higher, the sea floor stays as it was bit for bit, and the
/// highest ground rises by less than the mountain factor, since only the mountains grow.
#[test]
fn weaker_pull_raises_the_mountains() {
    let (home, light) = (
        Globe::new(42, 6000.0, 22.0),
        Globe::new(42, 6000.0, 22.0 / 1.44),
    );
    let (mut top, mut high) = (f32::MIN, f32::MIN);
    for dir in sphere_dirs(20_000) {
        let (a, b) = (home.height(dir), light.height(dir));
        if a < -3.0 {
            assert_eq!(a.to_bits(), b.to_bits(), "{dir}");
        }
        top = top.max(a);
        high = high.max(b);
    }
    let rise = high / top;
    assert!(
        (1.05..=1.2).contains(&rise),
        "summits {top} m at home, {high} m under a weaker pull"
    );
}

/// The legacy plane follows none of the physics: a heavy, a light, a small and a large planet of one seed
/// give the same plane bit for bit.
#[test]
fn the_plane_ignores_the_physics() {
    let home = Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY);
    let others = [
        Globe::new(7, 6000.0, 9.8),
        Globe::new(7, 6000.0, 80.0),
        Globe::new(7, 40_000.0, 22.0),
    ];
    for (x, z) in (0..600).map(|i| (i as f32 * 37.3 - 9000.0, i as f32 * -29.1 + 7000.0)) {
        for other in &others {
            assert_eq!(
                home.height_flat(x, z).to_bits(),
                other.height_flat(x, z).to_bits()
            );
            assert_eq!(home.biome_flat(x, z), other.biome_flat(x, z));
            assert_eq!(home.surface_flat(x, z), other.surface_flat(x, z));
        }
    }
}

/// A planet twice the size has the same continents at the same directions: its continent measure is the
/// home one bit for bit.
#[test]
fn continents_keep_their_angular_size() {
    let (home, large) = (Globe::new(42, 6000.0, 22.0), Globe::new(42, 12_000.0, 22.0));
    for dir in sphere_dirs(4000) {
        let at = |g: &Globe| g.shape.base((dir * g.radius()).to_array());
        assert_eq!(at(&home).to_bits(), at(&large).to_bits(), "{dir}");
    }
}

/// Returns the land share, the mean land height and the 90th and 99th percentile heights of `globe` over
/// 20,000 sites.
fn measure(globe: &Globe) -> [f32; 4] {
    let mut h: Vec<f32> = sphere_dirs(20_000).map(|d| globe.height(d)).collect();
    let land: Vec<f32> = h.iter().copied().filter(|&v| v > SEA_LEVEL).collect();
    h.sort_by(f32::total_cmp);
    let q = |f: f32| h[((h.len() - 1) as f32 * f) as usize];
    let share = land.len() as f32 / h.len() as f32;
    [
        share,
        land.iter().sum::<f32>() / land.len() as f32,
        q(0.9),
        q(0.99),
    ]
}

/// The home ground keeps its character: land share, mean land height and high percentiles within a few
/// percent of what they were before rivers and the climate, for three seeds.
#[test]
fn the_home_ground_keeps_its_character() {
    let before: [(u64, [f32; 4]); 3] = [
        (42, [0.3522, 9.947, 13.496, 33.987]),
        (1, [0.3516, 9.859, 13.266, 33.488]),
        (7, [0.3475, 9.866, 13.083, 31.784]),
    ];
    for (seed, was) in before {
        let now = measure(&Globe::new(seed, 6000.0, 22.0));
        let near = |a: f32, b: f32, share: f32| (a - b).abs() <= share * b.abs().max(1.0);
        assert!(
            near(now[0], was[0], 0.02),
            "seed {seed} land {now:?} against {was:?}"
        );
        assert!(
            near(now[1], was[1], 0.04),
            "seed {seed} mean {now:?} against {was:?}"
        );
        assert!(
            near(now[2], was[2], 0.04) && near(now[3], was[3], 0.04),
            "seed {seed} {now:?} {was:?}"
        );
    }
}
