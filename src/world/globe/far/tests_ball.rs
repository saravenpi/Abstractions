//! Ball tests: the maps of the surface the sky draws hold one texel per cell of the coarse nodes of
//! the world, keep the colours of those nodes block for block, so the planet looks the same on
//! both sides of the hand-over, and carry the sea and its glow.

use std::sync::Arc;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::ball::texel;
use super::{Map, NODE_CELLS, SIDES, ball, face};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Terrain;
use crate::world::colormap;
use crate::world::globe::{ChunkId, Face, Globe};
use crate::world::mesh::{Frame, fill_node};
use crate::world::shapes::FlatMesh;

/// Returns a 6 km globe of `class` whose sea covers the `ocean` share of it.
fn globe(class: PlanetClass, ocean: f64) -> Arc<Globe> {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    Arc::new(Globe::new(7, 6000.0, 9.8).with_record(record))
}

/// Returns the six maps of `globe` at `side` texels.
fn maps(globe: &Arc<Globe>, side: usize) -> Vec<Map> {
    Face::ALL
        .map(|at| face(globe, at, side))
        .into_iter()
        .collect()
}

/// Returns the mean linear colour of the coarse node `id` the world draws, each triangle counted
/// by its area.
fn node_mean(globe: &Arc<Globe>, id: ChunkId) -> Vec3 {
    let terrain = Terrain::on(globe.clone(), globe.face_chart(id.face));
    let mut mesh = FlatMesh::default();
    fill_node(&mut mesh, &terrain, (id, NODE_CELLS), &Frame::PLANET, false);
    let mesh = mesh.into_mesh();
    let (
        Some(VertexAttributeValues::Float32x3(points)),
        Some(VertexAttributeValues::Float32x2(uvs)),
    ) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_UV_0),
    )
    else {
        return Vec3::ZERO;
    };
    let image = colormap::image();
    let (mut sum, mut total) = (Vec3::ZERO, 0.0);
    for (corners, uv) in points.as_chunks::<3>().0.iter().zip(uvs.iter().step_by(3)) {
        let [a, b, c] = corners.map(Vec3::from);
        let area = (b - a).cross(c - a).length();
        sum += texel(&image, *uv) * area;
        total += area;
    }
    sum / total
}

/// Returns the mean colour of the block of `map` that node `(x, y)` of `nodes` a side covers.
fn block_mean(map: &Map, nodes: usize, (x, y): (usize, usize)) -> Vec3 {
    let cells = map.side / nodes;
    let rows = (y * cells..(y + 1) * cells)
        .flat_map(|j| (x * cells..(x + 1) * cells).map(move |i| (i, j)));
    let sum: Vec3 = rows
        .map(|(i, j)| map.texels[j * map.side + i].truncate())
        .sum();
    sum / (cells * cells) as f32
}

/// A map has one texel per cell, every one a colour and a share of sea between 0 and 1.
#[test]
fn a_map_has_a_texel_per_cell() {
    let globe = globe(PlanetClass::Island, 0.6);
    let map = face(&globe, Face::PosY, SIDES[0]);
    assert_eq!((map.side, map.texels.len()), (64, 64 * 64));
    let fine = |t: &Vec4| t.is_finite() && t.min_element() >= 0.0 && t.max_element() <= 1.0;
    assert!(map.texels.iter().all(fine));
    assert!(map.area > 0.0);
}

/// On every solid class, each block of the map at the hand-over keeps the mean colour of the coarse
/// node of the world it stands for within 0.02 in linear light: the planet the sky draws has the
/// colours of the ground the world draws, in the same places.
#[test]
fn the_map_keeps_the_colours_of_the_near_nodes() {
    let solid = |class: &PlanetClass| *class != PlanetClass::GasGiant;
    for class in PlanetClass::ALL.into_iter().filter(solid) {
        let globe = globe(class, 0.5);
        let map = face(&globe, Face::PosZ, SIDES[1]);
        let nodes = SIDES[1] / NODE_CELLS;
        for (x, y) in [(0, 0), (3, 0), (0, 3), (1, 2), (3, 3)] {
            let id = ChunkId {
                face: Face::PosZ,
                level: 2,
                x: x as u32,
                y: y as u32,
            };
            let (near, far) = (node_mean(&globe, id), block_mean(&map, nodes, (x, y)));
            let off = (near - far).abs().max_element();
            assert!(off < 0.02, "{class:?} node {x} {y}: off by {off}");
        }
    }
}

/// The ball knows its sea: none on a dry world, some on a wet one, and the light a sea of molten rock
/// gives off, which water does not.
#[test]
fn the_ball_carries_the_sea_and_its_glow() {
    let lava = globe(PlanetClass::Lava, 0.4);
    let molten = ball(&lava, &maps(&lava, SIDES[0]));
    let look = PlanetClass::Lava.look();
    assert!(molten.sea > 0.01 && molten.sea < 0.9, "{}", molten.sea);
    assert_eq!(molten.glow, look.sea * look.glow);
    let dry = globe(PlanetClass::Lava, 0.0);
    let dry = ball(&dry, &maps(&dry, SIDES[0]));
    assert_eq!((dry.sea, dry.glow), (0.0, Vec3::ZERO));
    let wet = globe(PlanetClass::Island, 0.6);
    let home = ball(&wet, &maps(&wet, SIDES[0]));
    assert!(home.sea > 0.2 && home.sea < 0.95, "{}", home.sea);
    assert_eq!(home.glow, Vec3::ZERO);
    assert!(home.mean.z > home.mean.x, "{}", home.mean);
}

/// Prints what each level of a ball costs to bake on one thread: run with `-- --ignored --nocapture`.
#[test]
#[ignore = "a measurement, not a check"]
fn ball_cost() {
    let globe = globe(PlanetClass::Island, 0.6);
    for side in SIDES {
        let start = std::time::Instant::now();
        let map = face(&globe, Face::PosX, side);
        println!(
            "{side} texels a side: one face {:?}, six faces about {:?}",
            start.elapsed(),
            start.elapsed() * 6
        );
        assert_eq!(map.texels.len(), side * side);
    }
}
