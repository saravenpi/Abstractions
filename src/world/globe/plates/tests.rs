//! Tests of the plates: the same seed gives the same plates, the crust follows the low share of the planet,
//! mountains stand near converging edges and nothing steps where plates meet.

use bevy::math::DVec3;

use super::*;
use crate::universe::PlanetRecord;
use crate::world::globe::tests_record::sphere_dirs;

/// Radius of the Earth in metres.
pub(super) const EARTH: f64 = 6.371e6;

/// Returns the plates of planet `seed`, as large, heavy and old as the Earth, with the home record.
pub(super) fn earth(seed: u64) -> Plates {
    let body = [EARTH, 9.81];
    let interior = Interior::of(seed, body, &PlanetRecord::TEMPERATE, 4.5);
    Plates::new(seed, EARTH, interior)
}

/// Returns true when two of the plates near `dir` close in along an edge within `reach` radians of it,
/// measured where the plates are looked up.
fn closes_within(plates: &Plates, dir: DVec3, reach: f64) -> bool {
    let w = warp::warped(plates.seed, dir);
    let near = plates.near(w);
    let mut pairs = (0..near.count).flat_map(|a| (a + 1..near.count).map(move |b| (a, b)));
    pairs.any(|(a, b)| {
        let (i, j) = (near.ids[a], near.ids[b]);
        let t = (near.dots[a] - near.dots[b]) / plates.chord(i, j);
        t.abs() < reach && Boundary::of(plates.closing(w, i, j)) == Boundary::Convergent
    })
}

/// Returns true when a plate other than the one that holds `dir` lies within `metres` of it on the ground
/// of an Earth, looked for on two rings of sixteen points.
fn edge_within(plates: &Plates, dir: DVec3, metres: f64) -> bool {
    let own = plates.sample(dir).plate;
    let (east, north) = dir.any_orthonormal_pair();
    (0..32).any(|k| {
        let angle = std::f64::consts::TAU * f64::from(k % 16) / 16.0;
        let reach = metres / EARTH * f64::from(1 + k / 16) * 0.5;
        let at = dir + (east * angle.cos() + north * angle.sin()) * reach;
        plates.sample(at.normalize()).plate != own
    })
}

/// The same seed gives the same plates and the same samples, and another seed gives other ground on
/// most of the sphere.
#[test]
fn the_same_seed_gives_the_same_plates() {
    let (a, b, other) = (earth(7), earth(7), earth(8));
    assert_eq!(a, b);
    let mut differ = 0;
    for dir in sphere_dirs(2000) {
        assert_eq!(a.sample(dir), b.sample(dir), "{dir}");
        differ += usize::from(a.sample(dir) != other.sample(dir));
    }
    assert!(
        differ > 1500,
        "seeds 7 and 8 agree on {} sites",
        2000 - differ
    );
}

/// A planet has 8 to 20 plates, each holds a share of the sphere, and the share of continental plates is
/// the share of the planet that is not low ground, to the nearest plate.
#[test]
fn plates_cover_the_sphere_and_carry_both_crusts() {
    for seed in [1, 7, 42, 99] {
        let plates = earth(seed);
        let n = plates.count();
        assert!((COUNT.0..=COUNT.1).contains(&n), "seed {seed}: {n} plates");
        let mut held = vec![0u32; n];
        let mut lands = vec![false; n];
        for s in sphere_dirs(20_000).map(|d| plates.sample(d)) {
            held[usize::from(s.plate)] += 1;
            lands[usize::from(s.plate)] = s.crust == Crust::Continental;
            assert!((-1.0..=1.0).contains(&s.land) && s.edge >= 0.0);
        }
        assert!(held.iter().all(|&h| h > 100), "seed {seed}: {held:?}");
        let want = ((1.0 - plates.interior().lowland) * n as f64).round() as usize;
        assert_eq!(lands.iter().filter(|&&l| l).count(), want, "seed {seed}");
    }
}

/// Mountains stand within 300 km of a converging edge: wherever the edges lift the ground by more than
/// 0.3 of the highest summit, two plates close in along an edge within the reach of a range, and another
/// plate lies within 300 km on the ground.
#[test]
fn mountains_stand_near_converging_edges() {
    let mut high = 0;
    for seed in [1, 7, 42] {
        let plates = earth(seed);
        let lifted = |d: &DVec3| plates.sample(*d).rise > 0.3;
        for dir in sphere_dirs(20_000).filter(lifted) {
            high += 1;
            assert!(closes_within(&plates, dir, RANGE), "seed {seed} at {dir}");
            assert!(edge_within(&plates, dir, 300_000.0), "seed {seed} at {dir}");
        }
    }
    println!("{high} of 60,000 sites are lifted past 0.3 of the highest summit");
    assert!(high > 10, "only {high} sites in mountains");
}

/// Nothing steps where plates meet: a move of 64 m on an Earth changes the crust blend and what the edges
/// add by less than 0.004, 35 m of the highest summit, on edges, at junctions and inside the plates alike.
#[test]
fn the_ground_of_the_plates_is_continuous() {
    let plates = earth(42);
    for (k, dir) in sphere_dirs(20_000).enumerate() {
        let (east, north) = dir.any_orthonormal_pair();
        let angle = k as f64 * 0.7;
        let next = (dir + (east * angle.cos() + north * angle.sin()) * 1e-5).normalize();
        let (a, b) = (plates.sample(dir), plates.sample(next));
        assert!((a.land - b.land).abs() < 0.004, "crust steps at {dir}");
        assert!((a.rise - b.rise).abs() < 0.004, "ranges step at {dir}");
        assert!(
            (a.floor - b.floor).abs() < 0.004,
            "the sea floor steps at {dir}"
        );
        assert!((a.cone - b.cone).abs() < 0.004, "a volcano steps at {dir}");
    }
}
