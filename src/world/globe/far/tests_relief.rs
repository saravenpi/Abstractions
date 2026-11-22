//! Relief tests: each texel of a map carries the mean lean of the facets the world draws there, so
//! the ball the sky draws catches the light on its slopes as the flat shaded ground does, and the
//! flat sea leans nowhere.

use std::sync::Arc;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::{Map, NODE_CELLS, SIDES, face};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Terrain;
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

/// Returns the mean lean of the facets of the coarse node `id` the world draws, each counted by
/// its area: its normal less the direction up at its middle.
fn node_tilt(globe: &Arc<Globe>, id: ChunkId) -> Vec3 {
    let terrain = Terrain::on(globe.clone(), globe.face_chart(id.face));
    let mut mesh = FlatMesh::default();
    fill_node(&mut mesh, &terrain, (id, NODE_CELLS), &Frame::PLANET, false);
    let mesh = mesh.into_mesh();
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Vec3::ZERO;
    };
    let (mut sum, mut total) = (Vec3::ZERO, 0.0);
    for corners in points.as_chunks::<3>().0 {
        let [a, b, c] = corners.map(Vec3::from);
        let normal = (b - a).cross(c - a);
        sum += normal - (a + b + c).normalize() * normal.length();
        total += normal.length();
    }
    sum / total
}

/// Returns the mean tilt of the block of `map` that node `(x, y)` of `nodes` a side covers.
fn block_tilt(map: &Map, nodes: usize, (x, y): (usize, usize)) -> Vec3 {
    let cells = map.side / nodes;
    let rows = (y * cells..(y + 1) * cells)
        .flat_map(|j| (x * cells..(x + 1) * cells).map(move |i| (i, j)));
    let sum: Vec3 = rows
        .map(|(i, j)| map.tilts[j * map.side + i].truncate())
        .sum();
    sum / (cells * cells) as f32
}

/// Each block of the map at the hand-over leans as the coarse node of the world it stands for, to
/// within 0.01, on every solid class.
#[test]
fn the_map_leans_as_the_near_nodes() {
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
            let (near, far) = (node_tilt(&globe, id), block_tilt(&map, nodes, (x, y)));
            let off = (near - far).abs().max_element();
            assert!(off < 0.01, "{class:?} node {x} {y}: off by {off}");
        }
    }
}

/// On a world of land and sea some land texels lean by more than a degree, every texel that is all
/// sea lies level, and no tilt is larger than a unit vector.
#[test]
fn the_land_leans_and_the_sea_lies_level() {
    let globe = globe(PlanetClass::Island, 0.6);
    let map = face(&globe, Face::PosY, SIDES[1]);
    let pairs = map.texels.iter().zip(&map.tilts);
    let (mut wet, mut steep) = (0, 0);
    for (texel, tilt) in pairs {
        let lean = tilt.truncate().length();
        assert!(
            lean.is_finite() && lean < std::f32::consts::SQRT_2 && tilt.w == 0.0,
            "{tilt}"
        );
        if texel.w >= 1.0 {
            wet += 1;
            assert!(lean < 5.0e-3, "a sea texel leans by {lean}");
        }
        steep += usize::from(texel.w <= 0.0 && lean > 0.0175);
    }
    assert!(wet > 0 && steep > 0, "{wet} sea texels, {steep} steep ones");
}
