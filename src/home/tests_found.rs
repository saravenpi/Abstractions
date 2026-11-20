//! Tests of where the base is founded once biomes follow the climate: around a mild northern
//! latitude, on firm ground out of the heart of a taiga or a desert.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Base;
use super::ground::{firm, harsh, spread};
use super::site::PASSES;
use super::tests::{base, homeworld};
use super::tune::STARTS;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Returns the base the search of 0.8 found: near the spawn of the centre of the north face, the
/// pole of a world tilted on its spin axis, or nothing when no ground near that spawn took it.
fn polar(seed: u64) -> Option<Base> {
    let (globe, home) = homeworld(seed);
    let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let at = north.find_spawn();
    let spawn = north.embed(at.x, at.y, at.z);
    [PASSES[0], PASSES[2]]
        .iter()
        .find_map(|pass| Base::near(&globe, home, spawn, *pass))
}

/// Returns the latitude in degrees of the planet position `at`.
fn latitude(at: DVec3) -> f64 {
    at.normalize().y.asin().to_degrees()
}

/// On seeds 0 to 50 the base stands on firm ground out of the heart of a taiga or a desert, off
/// the polar cap, and a new game lands on dry ground off the poles within reach of it. The test prints
/// how many bases left the place the search of 0.8 gave them.
#[test]
fn every_seed_founds_its_base_on_good_ground() {
    let mut moved = Vec::new();
    for seed in 0..=50 {
        let (found, globe) = base(seed);
        let landing = Base::landing(&globe);
        assert!(
            latitude(landing).abs() < 60.0,
            "seed {seed}: lands near a pole"
        );
        assert!(
            globe.height(landing.normalize()) > globe.sea_level(),
            "seed {seed}: lands at sea"
        );
        assert!(
            landing.distance(found.centre) < globe.radius(),
            "seed {seed}: lands far from the base"
        );
        let lot = found.lot(&globe);
        assert!(firm(lot.view(), Vec2::ZERO), "seed {seed}: soft ground");
        let view = Terrain::near(&globe, landing);
        let there = view.locate(landing);
        assert!(
            !harsh(lot.view(), Vec2::ZERO) || harsh(&view, Vec2::new(there.x, there.z)),
            "seed {seed}: in a harsh heart, off a landing that is not"
        );
        let at = latitude(found.up);
        assert!(at.abs() < 75.0, "seed {seed}: base at {at} degrees");
        if polar(seed).is_none_or(|old| old.centre.distance(found.centre) > 1.0) {
            moved.push(seed);
        }
    }
    println!("BASES MOVED {} of 51: {moved:?}", moved.len());
}

/// The fallback ring lies on the mild latitude, spread around the spin axis, and the seed turns it.
#[test]
fn the_ring_of_starts_is_a_mild_latitude() {
    let dirs: Vec<DVec3> = spread(7).collect();
    assert_eq!(dirs.len(), STARTS.1 as usize);
    for dir in &dirs {
        assert!((dir.length() - 1.0).abs() < 1e-9);
        assert!((dir.y.asin().to_degrees() - f64::from(STARTS.0)).abs() < 1e-6);
    }
    assert!((0..dirs.len()).all(|i| (0..i).all(|j| dirs[i].distance(dirs[j]) > 0.5)));
    assert_eq!(spread(7).next(), spread(7).next());
    assert!(spread(7).next() != spread(8).next());
}

/// Returns true when sea lies within 2 km of the base centre of `found` on `globe`, sampled on circles.
fn on_a_coast(found: &Base, globe: &Arc<Globe>) -> bool {
    let lot = found.lot(globe);
    let circles = [250.0_f32, 500.0, 1000.0, 2000.0];
    let points = circles.iter().flat_map(|r| {
        (0..32).map(move |k| Vec2::from_angle(k as f32 / 32.0 * std::f32::consts::TAU) * *r)
    });
    points
        .into_iter()
        .any(|p| lot.view().height(p.x, p.y) <= 0.0)
}

/// On the seeds whose home planet has a sea, the base lands on a coast: the starts are shores of the
/// drainage grid at a mild latitude, and sea lies within 2 km of the base.
#[test]
fn the_base_lands_on_a_coast() {
    let mut coasts = 0;
    for seed in 0..=20 {
        let (found, globe) = base(seed);
        if !globe.has_sea() {
            continue;
        }
        coasts += 1;
        assert!(
            on_a_coast(&found, &globe),
            "seed {seed}: no sea within 2 km of the base"
        );
        let degrees = latitude(found.up).abs();
        assert!(degrees < 50.0, "seed {seed}: base at {degrees} degrees");
    }
    assert!(coasts > 0, "no seed has a home planet with a sea");
}
