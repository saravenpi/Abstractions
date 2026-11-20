//! Tests of the spawn rules on the two kinds of chart: the legacy plane keeps to grass and sand, as it did
//! before the planets had cold lands, and a sphere also takes flat snow and rock.

use std::sync::Arc;

use bevy::prelude::*;

use super::Terrain;
use crate::universe::Universe;
use crate::world::cpu::busy;
use crate::world::globe::Face;
use crate::world::{Globe, Surface};

/// Returns how many points of a 3 km grid around the origin of `terrain` lie on snow or rock, and how many
/// of those are good spawns.
fn bare(terrain: &Terrain) -> (usize, usize) {
    let grid = (-75..75).flat_map(|i| (-75..75).map(move |j| Vec2::new(i as f32, j as f32) * 20.0));
    let on = grid.filter(|p| matches!(terrain.surface(p.x, p.y), Surface::Snow | Surface::Rock));
    let good: Vec<bool> = on.map(|p| terrain.good_spawn(p.x, p.y)).collect();
    (good.len(), good.iter().filter(|g| **g).count())
}

/// On the legacy plane no point on snow or rock is a spawn, among hundreds of them over three seeds, so the
/// flat world keeps its spawns; around the cold pole of a globe some are.
#[test]
fn only_a_sphere_spawns_on_snow_or_rock() {
    for seed in [1, 2, 3] {
        let (on, good) = bare(&Terrain::new(seed));
        assert!(on > 100, "seed {seed} has {on} points on snow or rock");
        assert_eq!(good, 0, "seed {seed}: the plane spawns on snow or rock");
    }
    let globe = Arc::new(Globe::new(42, 6000.0, 22.0));
    let pole = Terrain::on(globe.clone(), globe.face_chart(Face::PosY)).around(0.0, 0.0);
    let (on, good) = bare(&pole);
    assert!(good > 0, "none of {on} points on snow or rock is a spawn");
}

/// The most processor time in seconds a spawn search may take once the drainage grid is built.
const SEARCH: f64 = 0.2;

/// On the home planets of 50 seeds the spawn search from the pole of the north face, which is often open
/// sea or ice on a large planet, finds dry ground every time, within 200 ms of processor time once the grid exists.
#[test]
fn every_seed_spawns_on_land_in_time() {
    for seed in 0..50 {
        let universe = Universe::new(seed);
        let globe = Arc::new(Globe::of(&universe.planet(universe.home())));
        globe.drainage();
        let view = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
        let start = busy();
        let site = view.spawn_site();
        let took = busy() - start;
        let site = site.unwrap_or_else(|| panic!("seed {seed}: no spawn"));
        assert!(
            globe.height(site.normalize()) > globe.sea_level(),
            "seed {seed}: spawns at sea"
        );
        assert!(took < SEARCH, "seed {seed}: the spawn took {took} s");
    }
}
