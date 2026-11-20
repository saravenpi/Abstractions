//! Tests of chunk ownership along the cube edges: every prop belongs to one chunk, none is doubled or dropped.

use std::sync::Arc;

use bevy::math::DVec3;

use super::zones::{Tile, Zone, coords};
use super::*;
use crate::world::globe::{ChunkId, Face, Globe};
use crate::world::mesh::leaf_coord;

/// Returns the quadtree node of a chunk.
fn node(tile: &Tile) -> ChunkId {
    let level = tile.terrain.globe().leaf_level();
    let at = tile.coord + IVec2::splat((1i32 << level) / 2);
    ChunkId {
        face: tile.face,
        level,
        x: at.x as u32,
        y: at.y as u32,
    }
}

/// Returns the chunk that touches `tile` across the edge of its face, if `tile` lies along one.
fn across(globe: &Arc<Globe>, tile: &Tile) -> Option<Tile> {
    let id = node(tile);
    let steps = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut beyond = steps.into_iter().filter_map(|(dx, dy)| id.neighbor(dx, dy));
    let other = beyond.find(|n| n.face != id.face)?;
    let face = Terrain::on(globe.clone(), globe.face_chart(other.face));
    Some(Tile::new(globe, other.face, leaf_coord(&face, other)))
}

/// Returns every pair of chunks of `globe` that touch across a cube edge and are both mostly land, once each.
fn pairs(globe: &Arc<Globe>) -> Vec<(Tile, Tile)> {
    let side = 1i32 << globe.leaf_level();
    let along = coords(Zone::Edge, side, side / 2);
    Face::ALL
        .into_iter()
        .flat_map(|face| along.iter().map(move |c| (face, *c)))
        .map(|(face, coord)| Tile::new(globe, face, coord))
        .filter(|a| a.dry() >= 15)
        .filter_map(|a| across(globe, &a).map(|b| (a, b)))
        .filter(|(a, b)| (a.face as u8) < (b.face as u8) && b.dry() >= 15)
        .collect()
}

/// Returns true when planet position `pos` lies in the quadtree node of `tile`: on its face and in its cell.
fn owned(tile: &Tile, pos: DVec3) -> bool {
    let id = node(tile);
    let (face, u, v) = Face::locate(pos.normalize());
    let side = (1u64 << id.level) as f64;
    let inside = |c: f64, cell: u32| {
        let part = (c + 1.0) * 0.5 * side - f64::from(cell);
        (-1e-9..1.0 + 1e-9).contains(&part)
    };
    face == id.face && inside(u, id.x) && inside(v, id.y)
}

/// Returns how many props of two chunks that touch across a cube edge stand within 2 m of it and how many
/// stand 2 to 6 m from it, after checking that each lies in its own node and that none stands in both chunks.
fn strips(globe: &Globe, a: &Tile, b: &Tile) -> [usize; 2] {
    let edge = (a.face.axes()[0] - b.face.axes()[0]).normalize();
    let reach = |pos: DVec3| pos.normalize().dot(edge).abs().asin() * globe.radius();
    let (on_a, on_b) = (a.scatter(), b.scatter());
    for (p, pos) in on_a.iter().filter(|(_, pos)| reach(*pos) < 2.0) {
        let twin = on_b.iter().any(|(_, other)| pos.distance(*other) < 0.01);
        assert!(
            !twin,
            "{:?} at {pos} stands on both sides of the edge",
            p.plant
        );
    }
    let mut counts = [0; 2];
    for (tile, (p, pos)) in on_a
        .iter()
        .map(|p| (a, p))
        .chain(on_b.iter().map(|p| (b, p)))
    {
        assert!(
            owned(tile, *pos),
            "{:?} at {pos} is outside its chunk",
            p.plant
        );
        counts[0] += usize::from(reach(*pos) < 2.0);
        counts[1] += usize::from((2.0..6.0).contains(&reach(*pos)));
    }
    counts
}

/// Along every land edge of a 3,000 m globe each prop lies in the node of its own chunk, no prop of one chunk
/// repeats a prop of the chunk across the edge, and the 2 m on either side of the edge hold as many props per
/// metre of width as the 4 m behind them: the reference a scatter that neither doubled nor dropped would give.
#[test]
fn edges_neither_double_nor_drop_props() {
    let globe = Arc::new(Globe::new(5, 3000.0, 22.0));
    let (mut near, mut behind) = (0, 0);
    for (a, b) in pairs(&globe) {
        let [close, far] = strips(&globe, &a, &b);
        near += close;
        behind += far;
    }
    let share = near as f64 * 2.0 / behind as f64;
    assert!(near > 300, "only {near} props within 2 m of an edge");
    assert!(
        (0.8..1.25).contains(&share),
        "edge strips hold {share} of the reference density"
    );
}
