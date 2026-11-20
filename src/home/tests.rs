//! Tests of the site and the named points of the base, on the home planet of many seeds.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Base;
use super::site::edge_distance;
use super::tune::{DRY, EDGE_CLEAR, FOOTPRINT, MAX_RELIEF, SHORE_CLEAR};
use crate::universe::{BodyId, Universe};
use crate::world::{Globe, Planet};

/// Returns the globe and the id of the home planet of the world of `seed`.
pub fn homeworld(seed: u64) -> (Arc<Globe>, BodyId) {
    let universe = Universe::new(seed);
    let home = universe.home();
    (Arc::new(Globe::of(&universe.planet(home))), home)
}

/// Returns the streamed planet resource for `body` of `universe`.
pub fn planet(universe: &Universe, body: BodyId) -> Planet {
    let spec = universe.planet(body);
    Planet {
        body,
        globe: Arc::new(Globe::of(&spec)),
        day_seconds: spec.day_seconds,
    }
}

/// Returns the base of the world of `seed` with the globe it stands on.
pub fn base(seed: u64) -> (Base, Arc<Globe>) {
    let (globe, home) = homeworld(seed);
    (Base::on(&globe, home), globe)
}

/// Returns the lowest and the highest ground on a 4 m grid within `radius` of the base centre.
fn span(base: &Base, globe: &Arc<Globe>, radius: f32) -> (f32, f32) {
    let lot = base.lot(globe);
    let steps = (radius / 4.0) as i32;
    let cells = (-steps..=steps).flat_map(|i| (-steps..=steps).map(move |j| (i, j)));
    let points = cells.map(|(i, j)| Vec2::new(i as f32, j as f32) * 4.0);
    points
        .filter(|p| p.length() <= radius)
        .map(|p| lot.view().height(p.x, p.y))
        .fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)))
}

/// The same seed gives the same base twice, and other seeds give other bases.
#[test]
fn site_is_deterministic_and_follows_the_seed() {
    let (first, globe) = base(1);
    assert_eq!(first, base(1).0);
    assert_eq!(first, Base::on(&globe, first.body));
    let others = [2, 3, 42].map(|seed| base(seed).0);
    assert!(
        others
            .iter()
            .all(|other| other.centre.distance(first.centre) > 1.0)
    );
    assert!(others[0].centre.distance(others[1].centre) > 1.0);
}

/// On 50 seeds the site is dry, within the slope limit, inland and clear of the cube edges.
#[test]
fn sites_are_dry_gentle_inland_and_off_the_edges() {
    for seed in 1..=50 {
        let (base, globe) = base(seed);
        let (low, high) = span(&base, &globe, FOOTPRINT as f32);
        assert!(
            low > DRY - 0.5,
            "seed {seed}: ground at {low} m under the base"
        );
        assert!(
            high - low < MAX_RELIEF + 1.0,
            "seed {seed}: relief {}",
            high - low
        );
        let shore = span(&base, &globe, FOOTPRINT as f32 + SHORE_CLEAR - 2.0).0;
        assert!(
            shore > 0.0,
            "seed {seed}: sea near the base, ground at {shore} m"
        );
        let edge = edge_distance(base.up, globe.radius());
        assert!(
            edge >= FOOTPRINT + EDGE_CLEAR,
            "seed {seed}: {edge} m from a cube edge"
        );
        assert!((base.centre.length() - globe.radius()) > f64::from(DRY) - 0.5);
    }
}

/// The distance to a cube edge is a quarter turn's eighth at a face centre and zero on an edge.
#[test]
fn edge_distance_spans_a_face() {
    let radius = 6000.0;
    let centre = edge_distance(DVec3::Y, radius);
    assert!((centre - radius * std::f64::consts::FRAC_PI_4).abs() < 1e-6);
    assert!(edge_distance(DVec3::new(1.0, 1.0, 0.3), radius).abs() < 1e-9);
    let near = edge_distance(DVec3::new(0.02, 1.0, 1.0 - 0.05), radius);
    assert!(near > 100.0 && near < 250.0, "{near}");
}
