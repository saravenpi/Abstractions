//! Tests that no site is cut by a cube edge, and that chunks near an edge list every site exactly once.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Terrain;
use super::globes::{RADII, faces, half_width};
use super::site::{CELL, Site, origins_near, sites_in_chunk, structure_in_cell};
use crate::world::CHUNK_SIZE;
use crate::world::globe::Chart;

/// Returns the distance in meters from the origin of `site` to the four edges of the cube face `face`: along
/// each edge, the nearest of 200 points, then the nearest of 200 points around that one.
fn to_edges(face: &Terrain, site: &Site) -> f64 {
    let Chart::Face { face: side, .. } = face.chart() else {
        return f64::MAX;
    };
    let here = face.up(site.origin.x, site.origin.y);
    let gap = |dir: DVec3| here.cross(dir).length().atan2(here.dot(dir)) * face.globe().radius();
    let along = |edge: (bool, f64)| {
        let at = |t: f64| {
            gap(if edge.0 {
                side.dir(edge.1, t)
            } else {
                side.dir(t, edge.1)
            })
        };
        let coarse = (0..=200).map(|i| f64::from(i) / 100.0 - 1.0);
        let best = coarse
            .min_by(|a, b| at(*a).total_cmp(&at(*b)))
            .unwrap_or(0.0);
        let fine = (0..=200).map(|i| best - 0.01 + f64::from(i) * 1e-4);
        fine.map(at).fold(f64::MAX, f64::min)
    };
    let edges = [(true, -1.0), (true, 1.0), (false, -1.0), (false, 1.0)];
    edges.map(along).into_iter().fold(f64::MAX, f64::min)
}

/// Returns over 2,000 cells of the chart `face`: every cell of a small face with a ring beyond its edges, or
/// the 23 rows of cells along both edges of each corner of a large face, one row of them beyond the edge.
fn cells(face: &Terrain) -> Vec<IVec2> {
    let n = (half_width(face) / CELL).ceil() as i32;
    let square = |m: i32| (-m..m).flat_map(move |x| (-m..m).map(move |y| IVec2::new(x, y)));
    if n <= 20 {
        return square(n + 10).collect();
    }
    let row = |v: i32| (n - 22..=n).contains(&(v.max(-v - 1)));
    square(n + 1).filter(|c| row(c.x) && row(c.y)).collect()
}

/// On globes of 3 and 40 km, over at least 2,000 cells of each face: every site keeps its reach plus 150 m
/// between its origin and the cube edges, sites do stand within 500 m of an edge, and none lies off its face.
#[test]
fn sites_keep_clear_of_cube_edges() {
    for radius in RADII {
        let (mut tried, mut close) = (0, 0);
        for face in &faces(7, radius) {
            let all = cells(face);
            assert!(all.len() >= 2000, "{} cells", all.len());
            for site in all.iter().filter_map(|c| structure_in_cell(face, *c)) {
                let room = to_edges(face, &site);
                let need = f64::from(site.kind.reach()) + 150.0;
                assert!(
                    room >= need - 0.5,
                    "{site:?} lies {room} m from an edge, needs {need}"
                );
                assert!(site.origin.abs().max_element() < half_width(face));
                tried += 1;
                close += usize::from(room < 500.0);
            }
        }
        assert!(
            tried >= 100 && close >= 5,
            "{tried} sites, {close} near an edge at {radius} m"
        );
    }
}

/// Along an edge and in a corner of a face of the 3 km globe: the chunks list every site of the cells they
/// cover exactly once, and the cleared points of each chunk hold the origin of every site in it.
#[test]
fn chunks_near_edges_list_each_site_once() {
    let face = &faces(7, RADII[0])[2];
    let chunks = (half_width(face) / CHUNK_SIZE) as i32;
    let span =
        |from: i32| (from..from + 12).flat_map(move |x| (chunks - 12..chunks).map(move |y| (x, y)));
    for (x, y) in span(-6).chain(span(chunks - 12)) {
        let coord = IVec2::new(x, y);
        let sites = sites_in_chunk(face, coord);
        let taken = origins_near(face, coord, 11.0);
        for site in &sites {
            assert!(
                taken.contains(&site.origin),
                "{site:?} is not cleared in its own chunk"
            );
            let cell = (site.origin / CELL).floor().as_ivec2();
            assert_eq!(structure_in_cell(face, cell), Some(*site));
        }
    }
    let cells = (6..=13).flat_map(|y| (-4..=13).map(move |x| IVec2::new(x, y)));
    let sites: Vec<Site> = cells.filter_map(|c| structure_in_cell(face, c)).collect();
    for site in &sites {
        let coord = (site.origin / CHUNK_SIZE).floor().as_ivec2();
        let listed = sites_in_chunk(face, coord)
            .iter()
            .filter(|s| *s == site)
            .count();
        assert_eq!(listed, 1, "{site:?} is listed {listed} times by its chunk");
    }
    assert!(!sites.is_empty(), "no site near the edge");
}
