//! Tests of the true-metre measure of a chunk: tries, density and the visible ground the globe scatter reads.

use std::sync::Arc;

use super::air::calm;
use super::zones::{Tile, Zone, chunks, globes, trio};
use super::*;
use crate::world::globe::Globe;
use crate::world::surface::classify;
use crate::world::{Biome, CHUNK_SIZE};

/// Tries per true square metre are those of the legacy plane within 1 percent at a face centre, at an edge
/// midpoint and next to a corner, and a metre of the measure is a metre on the ground along both axes.
#[test]
fn the_measure_is_in_true_metres() {
    for tile in trio() {
        let metric = Metric::of(&tile.terrain, chunk_center(tile.coord));
        let per_metre = metric.scaled(SMALL_TRIES) as f64 / tile.area();
        let flat = SMALL_TRIES as f64 / f64::from(CHUNK_SIZE * CHUNK_SIZE);
        let off = (per_metre / flat - 1.0).abs();
        assert!(off < 0.01, "{per_metre} tries per square metre");
        let ground = |metres: Vec2| {
            let o = metric.chart(metres);
            tile.planet(Vec3::new(o.x, 0.0, o.y)) - tile.planet(Vec3::ZERO)
        };
        let (east, south) = (ground(Vec2::X * 10.0), ground(Vec2::Y * 10.0));
        assert!((east.length() - 10.0).abs() < 0.1 && (south.length() - 10.0).abs() < 0.1);
        assert!(
            east.dot(south).abs() < 1.0,
            "the axes of the measure are not square"
        );
    }
}

/// Adds to `tally`, per biome, the props of `tile` that grow on grass and the square metres of grass it has.
fn tally_grassland(tile: &Tile, tally: &mut [(f64, f64); 6]) {
    let center = chunk_center(tile.coord);
    let grass = |local: Vec2| {
        let (w, t) = (center + local, &tile.terrain);
        (t.surface(w.x, w.y) == Surface::Grass).then(|| t.biome(w.x, w.y) as usize)
    };
    let cell = tile.area() / 256.0;
    let cells =
        (0..256).map(|i| Vec2::new((i % 16) as f32, (i / 16) as f32) * 4.0 - Vec2::splat(30.0));
    for biome in cells.filter_map(grass) {
        tally[biome].1 += cell;
    }
    for biome in tile.scatter().iter().filter_map(|(p, _)| grass(p.pos.xz())) {
        tally[biome].0 += 1.0;
    }
}

/// Returns, per biome, the props per square metre of grass over the full land chunks of `zone` of `globe`, or
/// nothing where the zone has less than one hectare of that grass.
fn grass_density(globe: &Arc<Globe>, zone: Zone) -> [Option<f64>; 6] {
    let mut tally = [(0.0, 0.0); 6];
    for tile in chunks(globe, zone, 25, 100) {
        tally_grassland(&tile, &mut tally);
    }
    tally.map(|(grown, area)| (area > 10_000.0).then(|| grown / area))
}

/// Like for like, the grass of one biome carries the same number of props per true square metre at a face
/// centre, at an edge midpoint and next to a corner, within 10 percent, on both globes. Biomes follow the
/// latitude, so each biome is compared between the zones that hold it: the tropics of a face centre on the
/// equator with an edge midpoint there, the temperate grass of an edge with that of a corner. The climate is
/// held still, as it differs by latitude between the places compared: only the chart is measured.
#[test]
fn density_is_even_across_a_face() {
    let mut compared = 0;
    let pairs = [
        (Zone::Centre, Zone::Edge),
        (Zone::Centre, Zone::Corner),
        (Zone::Edge, Zone::Corner),
    ];
    for globe in globes() {
        let density =
            [Zone::Centre, Zone::Edge, Zone::Corner].map(|z| calm(|| grass_density(&globe, z)));
        for (one, other) in pairs {
            let each = density[one as usize]
                .into_iter()
                .zip(density[other as usize]);
            for ((a, b), biome) in each
                .zip(Biome::ALL)
                .filter_map(|((a, b), z)| Some(((a?, b?), z)))
            {
                let off = (a / b - 1.0).abs();
                assert!(
                    off < 0.1,
                    "{biome:?} grass: {a} on {one:?}, {b} on {other:?}"
                );
                compared += 1;
            }
        }
    }
    assert!(compared >= 2, "only {compared} like for like comparisons");
}

/// The surface the globe scatter reads off the terrain mesh is the analytic one at nine land points in ten.
#[test]
fn mesh_surface_agrees_with_the_ground() {
    let globe = Arc::new(Globe::new(5, 3000.0, 22.0));
    let (mut same, mut all) = (0, 0);
    let zones = [Zone::Centre, Zone::Corner];
    for tile in zones.into_iter().flat_map(|z| chunks(&globe, z, 20, 6)) {
        let (t, center) = (&tile.terrain, chunk_center(tile.coord));
        let heights = chunk_heights(t, tile.coord, 1);
        let (relief, metric) = (Relief::new(tile.coord, &heights), Metric::of(t, center));
        for i in 0..400 {
            let w = center + Vec2::new((i % 20) as f32, (i / 20) as f32) * 3.1 - Vec2::splat(29.5);
            let (height, grade) = relief.at(w);
            let off = (height - t.mesh_height(w.x, w.y)).abs();
            assert!(off < 2e-3, "relief is {off} m off the mesh at {w}");
            let seen = classify(height, metric.slope(grade), t.biome(w.x, w.y));
            same += usize::from(height > 0.0 && seen == t.surface(w.x, w.y));
            all += usize::from(height > 0.0);
        }
    }
    assert!(
        all > 2000 && same * 10 >= all * 9,
        "{same} of {all} points agree"
    );
}
