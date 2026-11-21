//! Terrain tests: determinism and the shape of the world over 4 km for several seeds.

use std::collections::HashSet;

use super::*;
use crate::world::SEA_LEVEL;
use crate::world::globe::Face;

const SEEDS: [u64; 4] = [1, 2, 3, 42];

/// Returns a regular grid of points covering the 4 km square around the origin.
fn grid(step: f32) -> Vec<(f32, f32)> {
    let n = (4000.0 / step) as i32;
    (0..n)
        .flat_map(|i| (0..n).map(move |j| (i as f32 * step - 2000.0, j as f32 * step - 2000.0)))
        .collect()
}

/// The same seed gives the same heights and biomes and a different seed does not.
#[test]
fn heights_are_deterministic() {
    let (a, b, c) = (Terrain::new(7), Terrain::new(7), Terrain::new(8));
    let pts = [(0.0, 0.0), (130.5, -77.2), (-900.0, 412.0)];
    assert!(pts.iter().all(|&(x, z)| a.height(x, z) == b.height(x, z)));
    assert!(pts.iter().all(|&(x, z)| a.biome(x, z) == b.biome(x, z)));
    assert!(pts.iter().any(|&(x, z)| a.height(x, z) != c.height(x, z)));
}

/// The mesh height equals the true height on grid vertices and stays between its corners inside a cell.
#[test]
fn mesh_height_follows_the_grid() {
    let t = Terrain::new(3);
    for &(x, z) in &[(0.0, 0.0), (128.0, -64.0), (-902.0, 414.0)] {
        assert_eq!(t.mesh_height(x, z), t.height(x, z));
        let corners =
            [(0.0, 0.0), (2.0, 0.0), (0.0, 2.0), (2.0, 2.0)].map(|(a, b)| t.height(x + a, z + b));
        let (lo, hi) = corners
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        for &(a, b) in &[(0.5, 0.5), (1.5, 1.2), (0.3, 1.9)] {
            let m = t.mesh_height(x + a, z + b);
            assert!(m >= lo - 1e-3 && m <= hi + 1e-3, "{m} outside {lo}..{hi}");
        }
    }
}

/// Every seed has mostly sea, most biomes, tall peaks, deep trenches, shallows and reefs.
#[test]
fn world_has_sea_biomes_peaks_and_reefs() {
    for seed in SEEDS {
        let t = Terrain::new(seed);
        let pts = grid(40.0);
        let heights: Vec<f32> = pts.iter().map(|p| t.height(p.0, p.1)).collect();
        let sea = heights.iter().filter(|&&h| h <= SEA_LEVEL).count() as f32 / pts.len() as f32;
        let surfaces: HashSet<Surface> = pts.iter().map(|p| t.surface(p.0, p.1)).collect();
        let biomes: HashSet<Biome> = pts
            .iter()
            .zip(&heights)
            .filter(|(_, h)| **h > SEA_LEVEL)
            .map(|(p, _)| t.biome(p.0, p.1))
            .collect();
        let top = heights.iter().fold(f32::MIN, |a, &b| a.max(b));
        let bottom = heights.iter().fold(f32::MAX, |a, &b| a.min(b));
        assert!((0.55..0.80).contains(&sea), "seed {seed} sea {sea}");
        assert!(biomes.len() >= 5, "seed {seed} biomes {biomes:?}");
        assert!(
            top > 60.0 && bottom < -25.0,
            "seed {seed} range {bottom} {top}"
        );
        assert!(surfaces.contains(&Surface::Shallows), "seed {seed}");
        assert!(surfaces.contains(&Surface::Reef), "seed {seed}");
    }
}

/// The spawn sits inside a real island and no cold land touches a desert, on the flat legacy plane
/// whose sea is at [`SEA_LEVEL`], not on the home planet of the seed, whose sea is its own.
#[test]
fn spawn_is_inland_and_biomes_have_neighbours() {
    for seed in SEEDS {
        let t = Terrain::legacy(seed);
        let p = t.find_spawn();
        let dry = (0..16)
            .filter(|&a| {
                let d = Vec2::from_angle(a as f32 * std::f32::consts::TAU / 16.0) * 70.0;
                t.height(p.x + d.x, p.z + d.y) > SEA_LEVEL
            })
            .count();
        assert!(dry >= 12, "seed {seed} spawn {p} dry {dry}");
        let near_desert = |x: f32, z: f32| {
            (0..16).any(|a| {
                let d = Vec2::from_angle(a as f32 * std::f32::consts::TAU / 16.0) * 100.0;
                t.height(x + d.x, z + d.y) > SEA_LEVEL && t.biome(x + d.x, z + d.y) == Biome::Desert
            })
        };
        let clash = grid(80.0)
            .into_iter()
            .filter(|p| t.height(p.0, p.1) > SEA_LEVEL && t.biome(p.0, p.1) == Biome::Taiga)
            .any(|p| near_desert(p.0, p.1));
        assert!(!clash, "seed {seed} taiga touches desert");
    }
}

/// Returns true when a sea floor surface lies within 20 m of `p`.
fn touches_sea(t: &Terrain, p: (f32, f32)) -> bool {
    (0..8).any(|a| {
        let d = Vec2::from_angle(a as f32 * std::f32::consts::FRAC_PI_4) * 20.0;
        matches!(
            t.surface(p.0 + d.x, p.1 + d.y),
            Surface::Water | Surface::Shallows | Surface::Reef
        )
    })
}

/// Ponds exist, are ringed by land and never hold the spawn.
#[test]
fn ponds_are_inland_basins() {
    for seed in 1..=3 {
        let t = Terrain::legacy(seed);
        let mut blocks = HashSet::new();
        let mut checked = 0;
        for p in grid(6.0)
            .into_iter()
            .filter(|p| p.0.abs() < 1500.0 && p.1.abs() < 1500.0)
        {
            if t.surface(p.0, p.1) != Surface::Pond {
                continue;
            }
            blocks.insert(((p.0 / 300.0).floor() as i32, (p.1 / 300.0).floor() as i32));
            checked += 1;
            assert!(
                checked % 7 != 0 || !touches_sea(&t, p),
                "seed {seed} pond point {p:?} touches the sea"
            );
        }
        assert!(blocks.len() >= 3, "seed {seed} ponds {}", blocks.len());
        let s = t.find_spawn();
        assert_ne!(t.surface(s.x, s.z), Surface::Pond);
    }
}

/// Near the pole of a small globe, where the taiga lies under snow, the spawn stands on flat snow by the
/// coast instead of searching far away for grass.
#[test]
fn a_polar_spawn_stands_on_flat_snow() {
    let globe = Arc::new(Globe::new(1, 3000.0, 22.0));
    let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let p = north.find_spawn();
    assert_eq!(north.surface(p.x, p.z), Surface::Snow);
    assert!(
        Vec2::new(p.x, p.z).length() < 200.0,
        "spawn {p} far from the pole"
    );
    let rise = (north.height(p.x + 6.0, p.z) - north.height(p.x - 6.0, p.z)).abs();
    assert!(
        p.y > SEA_LEVEL + 2.0 && rise < 2.5,
        "spawn {p} wet or steep"
    );
}
