//! Tests of where the base is founded once biomes follow the climate: around a mild northern
//! latitude, on firm ground out of the heart of a taiga or a desert.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Base;
use super::ground::{firm, harsh, starts};
use super::site::SLACK;
use super::tests::{base, kodya};
use super::tune::STARTS;
use crate::world::Terrain;
use crate::world::globe::Face;

/// Returns the base the search of 0.8 found: near the spawn of the centre of the north face, the
/// pole of a world tilted on its spin axis, or nothing when no ground near that spawn took it.
fn polar(seed: u64) -> Option<Base> {
    let (globe, home) = kodya(seed);
    let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let at = north.find_spawn();
    let spawn = north.embed(at.x, at.y, at.z);
    SLACK
        .into_iter()
        .find_map(|slack| Base::near(&globe, home, spawn, slack))
}

/// On seeds 0 to 50 the base stands on firm ground out of the heart of a taiga or a desert, off
/// the polar cap. The test prints how many bases left the place the search of 0.8 gave them.
#[test]
fn every_seed_founds_its_base_on_good_ground() {
    let mut moved = Vec::new();
    for seed in 0..=50 {
        let (found, globe) = base(seed);
        let lot = found.lot(&globe);
        assert!(firm(lot.view(), Vec2::ZERO), "seed {seed}: soft ground");
        assert!(
            !harsh(lot.view(), Vec2::ZERO),
            "seed {seed}: in a harsh heart"
        );
        let latitude = found.up.y.asin().to_degrees();
        assert!(
            latitude.abs() < 75.0,
            "seed {seed}: base at {latitude} degrees"
        );
        if polar(seed).is_none_or(|old| old.centre.distance(found.centre) > 1.0) {
            moved.push(seed);
        }
    }
    println!("BASES MOVED {} of 51: {moved:?}", moved.len());
}

/// The starts lie on the mild latitude, spread around the spin axis, and the seed turns them.
#[test]
fn the_starts_ring_a_mild_latitude() {
    let dirs: Vec<DVec3> = starts(7).collect();
    assert_eq!(dirs.len(), STARTS.1 as usize);
    for dir in &dirs {
        assert!((dir.length() - 1.0).abs() < 1e-9);
        assert!((dir.y.asin().to_degrees() - f64::from(STARTS.0)).abs() < 1e-6);
    }
    assert!((0..dirs.len()).all(|i| (0..i).all(|j| dirs[i].distance(dirs[j]) > 0.5)));
    assert_eq!(starts(7).next(), starts(7).next());
    assert!(starts(7).next() != starts(8).next());
}
