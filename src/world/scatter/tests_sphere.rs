//! Tests of scatter on true spheres: ground contact, true spacing, stable output and budgets.

use super::spacing::{rock, room};
use super::zones::{Tile, tiles};
use super::*;
use crate::world::CHUNK_SIZE;
use crate::world::mesh::{Patch, chunk_patch};
use crate::world::propmesh::BIG;

/// Returns how far chunk frame point `at` lies above the terrain mesh `patch` along `up`.
fn lift(patch: &Patch, at: Vec3, up: Vec3) -> Option<f32> {
    let pos = patch.positions();
    patch.triangles().iter().find_map(|t| {
        let [a, b, c] = t.map(|i| pos[i as usize]);
        let (e1, e2, s) = (b - a, c - a, at - a);
        let (p, q) = (up.cross(e2), s.cross(e1));
        let det = e1.dot(p);
        let (u, v) = (s.dot(p) / det, up.dot(q) / det);
        (u >= -1e-4 && v >= -1e-4 && u + v <= 1.0 + 1e-4).then(|| -e2.dot(q) / det)
    })
}

/// On a 3,000 m and a 40,000 m globe every prop origin lies within 5 cm of the terrain mesh along the local up,
/// at face centres, edges and corners. Land rocks sit at or below it, never deeper than half their scale.
#[test]
fn props_sit_on_the_mesh_of_a_sphere() {
    let mut seen = 0;
    for (zone, tile) in tiles(15, 3) {
        let heights = chunk_heights(&tile.terrain, tile.coord, 1);
        let patch = chunk_patch(&tile.terrain, tile.coord, &heights);
        for (p, _) in tile.scatter() {
            let (at, up) = (tile.point(p.pos), tile.turn(p.pos) * Vec3::Y);
            let gap = lift(&patch, at, up).expect("the prop stands over its own chunk");
            let sink = f32::from(rock(p.plant) && p.pos.y > -0.5) * 0.5 * p.scale;
            let ok = gap <= 0.05 && gap >= -sink - 0.05;
            assert!(ok, "{:?} is {gap} m off the mesh on {zone:?}", p.plant);
            seen += 1;
        }
    }
    assert!(seen > 3000, "only {seen} props checked");
}

/// Returns, for every pair of big props of `tile`, how far apart they stand on the ground in true metres and
/// the distance their two rooms ask for.
fn gaps(tile: &Tile) -> Vec<(f32, f32)> {
    let all = tile.scatter().into_iter().map(|(p, _)| p);
    let big: Vec<Placement> = all.filter(|p| (p.plant as usize) < BIG).collect();
    let ground = |p: &Placement| tile.planet(Vec3::new(p.pos.x, 0.0, p.pos.z));
    let gap = |a: &Placement, b: &Placement| ground(a).distance(ground(b)) as f32;
    let pairs = (0..big.len()).flat_map(|i| (0..i).map(move |j| (i, j)));
    pairs
        .map(|(i, j)| (gap(&big[i], &big[j]), room(&big[i]) + room(&big[j])))
        .collect()
}

/// Big props keep the distance of their rooms in true metres on both globes, wherever the chart is squeezed.
/// Twelve chunks per zone are checked, as the climate thins the big props away from the heart of a biome.
#[test]
fn big_props_keep_true_distance_on_a_sphere() {
    let mut pairs = 0;
    for (zone, tile) in tiles(15, 12) {
        for (gap, rule) in gaps(&tile) {
            assert!(
                gap >= rule - 1e-3,
                "{gap} m for a rule of {rule} m on {zone:?}"
            );
            pairs += 1;
        }
    }
    assert!(pairs > 5000, "only {pairs} pairs checked");
}

/// Asserts that the budgets of scatter `s` follow the true area of `tile` and that nothing submerged pokes out
/// of the sea.
fn check_bounds(tile: &Tile, s: &Scatter) {
    let share = tile.area() / f64::from(CHUNK_SIZE * CHUNK_SIZE);
    let within = |count: usize, cap: usize, slack: f64| count as f64 <= cap as f64 * share + slack;
    let tris = s.grass.iter().map(small_tris).sum();
    assert!(within(s.props.len(), BIG_CAP, 1.0) && within(s.grass.len(), SMALL_CAP, 1.0));
    assert!(
        within(tris, TRI_BUDGET, 160.0),
        "{tris} triangles on {}",
        tile.coord
    );
    let wet = |p: &&Placement| p.pos.y < 0.0 && !rock(p.plant);
    for p in s.props.iter().chain(&s.grass).filter(wet) {
        let top = [p.plant.draft() * p.scale, p.scale + 1.0][usize::from(p.plant == Plant::Kelp)];
        assert!(
            p.pos.y + top <= 1e-3,
            "{:?} sticks out at {}",
            p.plant,
            p.pos.y
        );
    }
}

/// On a globe the same chunk always gets the same props, with or without its heights handed in, the budgets
/// follow its true area and nothing submerged pokes out of the sea.
#[test]
fn globe_scatter_is_stable_and_bounded() {
    let mut seen = 0;
    for (_, tile) in tiles(5, 4) {
        let s = scatter_chunk(&tile.terrain, tile.coord);
        let heights = chunk_heights(&tile.terrain, tile.coord, 1);
        assert_eq!(s, scatter_over(&tile.terrain, tile.coord, &heights));
        check_bounds(&tile, &s);
        seen += s.props.len() + s.grass.len();
    }
    assert!(seen > 3000, "only {seen} props checked");
}
