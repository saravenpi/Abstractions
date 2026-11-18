//! Sphere tests: land share at every radius, spawns on globe views and the cost of one height sample.

use std::f64::consts::PI;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use bevy::math::DVec3;

use crate::world::globe::{Face, Globe};
use crate::world::{SEA_LEVEL, Surface, Terrain};

/// Steepest rise over run the player walks on: a ground normal 0.64 high, about 50 degrees.
const WALKABLE_SLOPE: f32 = 1.2;

/// Returns `n` unit directions spread evenly over the sphere on a golden spiral.
fn sphere_dirs(n: u32) -> impl Iterator<Item = DVec3> {
    let golden = PI * (3.0 - 5f64.sqrt());
    (0..n).map(move |i| {
        let y = 1.0 - 2.0 * (f64::from(i) + 0.5) / f64::from(n);
        let (ring, turn) = ((1.0 - y * y).sqrt(), golden * f64::from(i));
        DVec3::new(ring * turn.cos(), y, ring * turn.sin())
    })
}

/// Between 15 and 45 percent of the globe is land for seeds 1 and 42 at 3, 6 and 40 km of radius.
#[test]
fn land_share_holds_at_every_radius() {
    for (seed, radius) in [1, 42]
        .into_iter()
        .flat_map(|s| [3e3, 6e3, 4e4].map(|r| (s, r)))
    {
        let globe = Globe::new(seed, radius, 22.0);
        let total = 6000;
        let land = sphere_dirs(total)
            .filter(|&d| globe.height(d) > SEA_LEVEL)
            .count();
        let share = land as f32 / total as f32;
        assert!(
            (0.15..0.45).contains(&share),
            "seed {seed} radius {radius} land {share}"
        );
    }
}

/// Returns the slope (rise over run) of the ground at chart position `(x, z)` of `view`, in metres.
fn slope_at(view: &Terrain, x: f32, z: f32) -> f32 {
    let site = view.around(x, z);
    let dx = (site.height(x + 1.5, z) - site.height(x - 1.5, z)) / 3.0;
    let dz = (site.height(x, z + 1.5) - site.height(x, z - 1.5)) / 3.0;
    dx.hypot(dz)
}

/// The spawn of a globe view is dry walkable beach or meadow on its own face, for 10 seeds, on the four
/// faces around the equator: the polar faces are taiga under snow now that biomes follow the latitude.
#[test]
fn globe_spawn_is_dry_and_walkable() {
    for seed in 1..=10 {
        let radius = [3000.0, 6000.0, 40_000.0][seed as usize % 3];
        let globe = Arc::new(Globe::new(seed, radius, 22.0));
        let face = [Face::PosX, Face::NegX, Face::PosZ, Face::NegZ][seed as usize % 4];
        let view = Terrain::on(globe.clone(), globe.face_chart(face));
        let p = view.find_spawn();
        assert_eq!(p.y, view.height(p.x, p.z));
        assert!(p.y > SEA_LEVEL + 1.5, "seed {seed} spawn {p} is wet");
        let surface = view.surface(p.x, p.z);
        assert!(
            matches!(surface, Surface::Grass | Surface::Sand),
            "seed {seed} {surface:?}"
        );
        let slope = slope_at(&view, p.x, p.z);
        assert!(slope < WALKABLE_SLOPE, "seed {seed} spawn slope {slope}");
        let arc = view.up(p.x, p.z).angle_between(view.up(0.0, 0.0)) * radius;
        assert!(
            arc < radius * 1.21,
            "seed {seed} spawn is {arc} m from the origin"
        );
        assert_eq!(view.find_spawn(), p);
    }
}

/// Returns the seconds taken by 100,000 calls of `sample`, each on its own position.
fn time_samples(sample: impl Fn(u32) -> f32) -> f64 {
    let start = Instant::now();
    let sum: f32 = (0..100_000).map(|i| sample(black_box(i))).sum();
    black_box(sum);
    start.elapsed().as_secs_f64()
}

/// 100,000 sphere height samples cost less than 2.5 times the same count of flat samples.
#[test]
fn sphere_height_is_not_slow() {
    let globe = Globe::new(42, 6000.0, 22.0);
    let dirs: Vec<DVec3> = sphere_dirs(100_000).collect();
    let flat =
        || time_samples(|i| globe.height_flat((i % 400) as f32 * 9.7, (i / 400) as f32 * 9.7));
    let ball = || time_samples(|i| globe.height(dirs[i as usize]));
    let rounds = (0..6).map(|_| (flat(), ball()));
    let (flat, ball) = rounds.fold((f64::MAX, f64::MAX), |best, run| {
        (best.0.min(run.0), best.1.min(run.1))
    });
    println!(
        "height_flat {:.0} ns, Globe::height {:.0} ns per call",
        flat * 1e4,
        ball * 1e4
    );
    assert!(
        ball < flat * 2.5,
        "sphere {ball} s against flat {flat} s for 100,000 samples"
    );
    assert!(ball < 2.0, "100,000 sphere samples took {ball} s");
}
