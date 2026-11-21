//! Tests of terrain meshes on the globe: shared edges, vertex radii and coarse node meshes.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::*;
use crate::world::Globe;
use crate::world::globe::{ChunkId, Face};

/// Returns a 6 km globe.
pub fn globe() -> Arc<Globe> {
    Arc::new(Globe::new(9, 6000.0, 22.0))
}

/// Returns the view of `globe` through the chart of `face`.
pub fn view(globe: &Arc<Globe>, face: Face) -> Terrain {
    Terrain::on(globe.clone(), globe.face_chart(face))
}

/// The vertices of a chunk patch lie at radius plus height from the planet centre within 1 mm,
/// and the mesh keeps two triangles per cell plus eight skirt triangles per cell of edge.
#[test]
fn chunk_vertices_sit_at_radius_plus_height() {
    let globe = globe();
    for (face, coord) in [
        (Face::PosY, IVec2::new(3, -5)),
        (Face::NegX, IVec2::new(-64, 63)),
    ] {
        let terrain = view(&globe, face);
        let heights = chunk_heights(&terrain, coord, 1);
        let patch = chunk_patch(&terrain, coord, &heights);
        let frame = Frame::at(&terrain, Span::leaf(&terrain, coord, 1).center());
        let n = GRID_POINTS - 1;
        for (i, j) in (0..=n).flat_map(|i| (0..=n).map(move |j| (i, j))) {
            let pos = frame.origin + frame.rot * patch.at(i, j).as_dvec3();
            let off = pos.length() - (globe.radius() + f64::from(heights[i][j]));
            assert!(off.abs() < 1e-3, "vertex {i},{j} is {off} m off");
            assert!((patch.at(i, j).length()) < 80.0);
        }
        let mesh = chunk_mesh(&terrain, coord, &heights, Lod::Near);
        assert_eq!(mesh.count_vertices(), (n * n * 2 + n * 8) * 3);
    }
}

/// A chunk frame has its origin on the sea level sphere and +Y pointing away from the centre.
#[test]
fn chunk_frame_is_tangent_to_the_sphere() {
    let globe = globe();
    for face in Face::ALL {
        let terrain = view(&globe, face);
        let frame = Frame::at(&terrain, Vec2::new(1234.0, -3210.0));
        assert!((frame.origin.length() - globe.radius()).abs() < 1e-6);
        let up = frame.rot * DVec3::Y;
        assert!(up.dot(frame.origin.normalize()) > 1.0 - 1e-12);
        assert!((frame.local(frame.origin + up * 5.0) - Vec3::Y * 5.0).length() < 1e-4);
    }
}

/// A coarse node mesh has the patch and its skirts, and never dips far below the flat sea.
#[test]
fn coarse_nodes_draw_a_flat_sea() {
    let globe = globe();
    let terrain = view(&globe, Face::PosZ);
    let id = ChunkId {
        face: Face::PosZ,
        level: 1,
        x: 1,
        y: 0,
    };
    let n = super::node::cells_of(&terrain, id);
    let mesh = node_mesh(&terrain, id);
    assert_eq!(mesh.count_vertices(), (n * n * 2 + n * 8) * 3);
    let frame = node_frame(&terrain, id);
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3());
    let surface = &positions.expect("the mesh has positions")[..n * n * 6];
    let lowest = surface
        .iter()
        .map(|p| (frame.origin + frame.rot * Vec3::from(*p).as_dvec3()).length() - globe.radius())
        .fold(f64::INFINITY, f64::min);
    assert!(
        (-1.6..-1.4).contains(&lowest),
        "lowest vertex at {lowest} m"
    );
}

/// A triangle dithered into the biome across a border takes its tint only when that biome
/// covers it with the same surface: a grass triangle never lands alone in desert sand, a sand
/// one never in tropical grass, while two grass biomes still blend.
#[test]
fn dithered_biomes_never_change_the_surface() {
    use crate::world::Biome;
    use crate::world::biome::ground_rgb;
    use crate::world::surface::classify;
    let ground = super::tris::ground;
    let sand = ground_rgb(classify(3.0, 0.1, Biome::Desert), Biome::Desert);
    assert_eq!(ground(3.0, 0.1, Biome::Desert, Biome::Tropical), sand);
    let grass = ground_rgb(classify(3.0, 0.1, Biome::Tropical), Biome::Tropical);
    assert_eq!(ground(3.0, 0.1, Biome::Tropical, Biome::Desert), grass);
    let blended = ground_rgb(classify(3.0, 0.1, Biome::Meadow), Biome::Forest);
    assert_eq!(ground(3.0, 0.1, Biome::Meadow, Biome::Forest), blended);
    assert_ne!(blended, ground(3.0, 0.1, Biome::Meadow, Biome::Meadow));
}
