//! Tests of the rooted face chart: far from the middle of a face of an Earth-size globe the ground is
//! read without steps, a tile is the same ground from every root, and the view follows the focus.

use std::sync::Arc;

use bevy::math::{DVec2, DVec3};
use bevy::prelude::*;

use super::Terrain;
use crate::world::CHUNK_SIZE;
use crate::world::globe::{BLOCK, Chart, ChunkId, Face, Globe};
use crate::world::mesh::leaf_coord;

/// Radius in metres of the Earth-size globe.
const EARTH: f64 = 6.371e6;

/// Metres between two samples of a run.
const STEP: f64 = 0.01;

/// Samples in a run.
const RUN: usize = 200;

/// Returns the Earth-size globe and the unit direction 3,000 km east of the middle of its +X face.
fn far_out() -> (Arc<Globe>, DVec3) {
    let globe = Arc::new(Globe::new(9, EARTH, 9.81));
    let u = 3.0e6 / (EARTH * std::f64::consts::FRAC_PI_4);
    (globe, Face::PosX.dir(u, 0.02))
}

/// Returns a chart position within a kilometre of `(0, 0)` of `view` where the ground rises or falls by
/// more than 2 cm per metre along x, so a run of samples there cannot be flat.
fn slope(view: &Terrain) -> Vec2 {
    let mut spots = (0..1000).map(|i| Vec2::new(i as f32, 37.0));
    let steep = |p: &Vec2| (view.height(p.x + 1.0, p.y) - view.height(p.x, p.y)).abs() > 0.02;
    spots.find(steep).expect("sloped ground within a kilometre")
}

/// Returns the heights `view` reads along x from chart position `from`, one every centimetre.
fn run(view: &Terrain, from: DVec2) -> Vec<f32> {
    let at = |i: usize| (from.x + i as f64 * STEP) as f32;
    (0..RUN)
        .map(|i| view.height(at(i), from.y as f32))
        .collect()
}

/// Returns how many samples of `heights` differ from the one before.
fn changes(heights: &[f32]) -> usize {
    heights.windows(2).filter(|w| w[0] != w[1]).count()
}

/// On an Earth-size globe, 3,000 km from the middle of a face, heights read 1 cm apart through the
/// rooted chart follow the ground: every one is within a millimetre of the ground read in 64 bits,
/// most samples differ from their neighbour and none jumps. The same run read through the chart
/// rooted in the middle of the face shows the 0.25 m steps of 32-bit positions there.
#[test]
fn earth_heights_are_smooth_far_from_the_face_centre() {
    let (globe, dir) = far_out();
    let view = Terrain::near(&globe, dir * EARTH);
    let Chart::Face { scale, origin, .. } = view.chart() else {
        panic!("a face chart");
    };
    assert!(view.locate(dir * EARTH).xz().length() < BLOCK as f32);
    let from = slope(&view).as_dvec2();
    let heights = run(&view, from);
    for (i, h) in heights.iter().enumerate() {
        let at = (origin + from + DVec2::X * (i as f64 * STEP)) / scale;
        let truth = globe.height(Face::PosX.dir(at.x, at.y));
        assert!(
            (h - truth).abs() < 1.0e-3,
            "sample {i}: {h} against {truth}"
        );
    }
    assert!(changes(&heights) > RUN * 3 / 4, "{}", changes(&heights));
    let jump = heights.windows(2).map(|w| (w[1] - w[0]).abs());
    assert!(jump.fold(0.0, f32::max) < 0.05);
    let middle = Terrain::on(globe.clone(), globe.face_chart(Face::PosX));
    let stepped = run(&middle, origin + from);
    assert!(changes(&stepped) < RUN / 5, "{}", changes(&stepped));
}

/// A leaf of the quadtree is the same ground through two roots of its face: its chunk coordinate
/// differs by the whole chunks between the roots, and its corner embeds at the same planet position.
#[test]
fn a_tile_is_the_same_ground_from_every_root() {
    let (globe, dir) = far_out();
    let near = Terrain::near(&globe, dir * EARTH);
    let moved = near.chart().origin() + DVec2::new(BLOCK, -2.0 * BLOCK);
    let other = Terrain::on(globe.clone(), near.chart().rooted(moved));
    let level = globe.leaf_level();
    let id = ChunkId {
        face: Face::PosX,
        level,
        x: (1u32 << level) / 2 + 39_000,
        y: (1u32 << level) / 2 + 12,
    };
    let (a, b) = (leaf_coord(&near, id), leaf_coord(&other, id));
    assert_eq!(a + near.shift(CHUNK_SIZE), b + other.shift(CHUNK_SIZE));
    assert_eq!(a - b, IVec2::new(60, -120));
    let corner = |view: &Terrain, c: IVec2| view.embed(c.x as f32 * 64.0, 0.0, c.y as f32 * 64.0);
    assert!(corner(&near, a).distance(corner(&other, b)) < 1.0e-6);
    let home = other.home(b, CHUNK_SIZE);
    assert_eq!(home.id, b + other.shift(CHUNK_SIZE));
    assert!(home.cell.min_element() >= 0 && home.cell.max_element() < 60);
    let back = home.cell.as_vec2() * 64.0 + home.offset;
    assert_eq!(back, b.as_vec2() * 64.0);
}

/// The shared view keeps its root while the focus stays within a block of it and is rooted again
/// near the focus beyond: the same face, whole blocks away, with the focus near the new origin.
#[test]
fn the_view_follows_the_focus_by_whole_blocks() {
    let (globe, dir) = far_out();
    let view = Terrain::near(&globe, dir * EARTH);
    let ahead = |metres: f32| view.up(metres, 0.0);
    assert!(view.followed(ahead(0.9 * BLOCK as f32)).is_none());
    let next = view
        .followed(ahead(1.2 * BLOCK as f32))
        .expect("a new root");
    let by = next.chart().origin() - view.chart().origin();
    assert_eq!(by, DVec2::new(BLOCK, 0.0));
    assert_eq!(view.chart().carried(&next.chart()), Some(-by));
    let at = next.chart().locate(ahead(1.2 * BLOCK as f32));
    assert!((at - Vec2::new(0.2 * BLOCK as f32, 0.0)).length() < 1.0e-2);
    let flat = Terrain::new(3);
    assert!(flat.followed(DVec3::Y).is_none());
    assert_eq!(flat.shift(CHUNK_SIZE), IVec2::ZERO);
}
