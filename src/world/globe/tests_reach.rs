//! Tests of the cells in reach: the cells gathered around one are picked by true distance on the sphere, so
//! nothing a flow or a crater can reach is left out at a face edge or a corner.

use super::Globe;
use super::atlas::{Cell, PitMap, STEPS, around};
use super::craters::{Crater, craters};
use super::cube::Face;
use super::tests_landform::world;
use crate::universe::PlanetClass;

/// Returns every cell of a planet of `n` cells per face side.
fn every(n: i32) -> Vec<Cell> {
    let on = |face| (0..n).flat_map(move |i| (0..n).map(move |j| Cell { face, i, j }));
    Face::ALL.into_iter().flat_map(on).collect()
}

/// Returns the smallest angle between any two of the corners, side middles and middles of cells `a` and `b`.
fn gap(a: Cell, b: Cell, n: i32) -> f64 {
    let spots = [0.0, STEPS as f64 * 0.5, STEPS as f64];
    let points = |c: Cell| {
        spots
            .into_iter()
            .flat_map(move |x| spots.map(|y| c.dir([x, y], n)))
    };
    let angles = points(a).flat_map(|p| points(b).map(move |q| p.angle_between(q)));
    angles.fold(f64::MAX, f64::min)
}

/// On globes of 3 and 6 km, the cells gathered around a cell at the middle of a face, on an edge and in a
/// corner hold every cell a point of which lies within 5 km of a point of it.
#[test]
fn cells_in_reach_are_picked_by_true_distance() {
    for radius in [3000.0, 6000.0] {
        let n = Globe::new(1, radius, 22.0).cells();
        let reach = 5000.0 / radius;
        for (i, j) in [(n / 2, n / 2), (0, n / 2), (0, 0), (n - 1, n - 1)] {
            let face = Face::PosX;
            let cell = Cell { face, i, j };
            let found = around(cell, n, reach);
            let near = every(n).into_iter().filter(|&c| gap(cell, c, n) <= reach);
            let missed: Vec<Cell> = near.filter(|c| !found.contains(c)).collect();
            assert!(missed.is_empty(), "{radius} m, {cell:?}: {missed:?}");
        }
    }
}

/// On a world of craters, the pit map of a cell keeps the craters of the cell and the few of its neighbours
/// that reach into it, not all those of the cells around, and the ground it gives is the sum of every crater
/// of the planet.
#[test]
fn pit_maps_keep_the_craters_that_reach_them() {
    let globe = world(PlanetClass::Barren, 0.0, 250.0);
    let n = globe.cells();
    let all: Vec<Crater> = every(n).iter().flat_map(|&c| craters(&globe, c)).collect();
    let face = Face::NegZ;
    for (i, j) in [(n / 2, n / 2), (0, n / 2), (0, 0)] {
        let cell = Cell { face, i, j };
        let (own, kept) = (
            craters(&globe, cell).len(),
            PitMap::build(&globe, cell).len(),
        );
        assert!(
            own <= kept && kept < 2 * own,
            "{kept} kept for {own} craters"
        );
        for (a, b) in (0..=8).flat_map(|a| (0..=8).map(move |b| (a, b))) {
            let p = cell.dir([a, b].map(|k| f64::from(k) * 2.0), n) * globe.radius();
            let sum: f32 = all.iter().map(|crater| crater.depth(p)).sum();
            assert!((globe.pitted(p) - sum).abs() < 1e-3, "{cell:?} at {a}, {b}");
        }
    }
}
