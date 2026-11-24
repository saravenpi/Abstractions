//! Tests for terrain crater stamping, shared chunk-edge agreement, and mesh darkening.

use std::sync::Arc;

use bevy::prelude::*;

use super::{DigStamp, Digs, MAX_DIG_DEPTH};
use crate::universe::Universe;
use crate::world::globe::Face;
use crate::world::mesh::{Frame, GRID_POINTS, Lod, Span, chunk_heights, chunk_mesh};
use crate::world::{Globe, Terrain};

/// Neighboring chunks straddling a crater agree identically along their shared edge vertices.
#[test]
fn dig_craters_match_across_chunk_edges_without_cracks() {
    let universe = Universe::new(11);
    let planet = universe.home();
    let globe = Arc::new(Globe::of(&universe.planet(planet)));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let mut digs = Digs::default();
    let edge_dir = Span::leaf(&terrain, IVec2::new(1, 0), GRID_POINTS - 1).dir(0, 16);
    digs.push(DigStamp {
        planet,
        dir: edge_dir.to_array(),
        radius: 3.0,
        depth: 1.5,
    });
    let mut left = chunk_heights(&terrain, IVec2::new(0, 0), 1);
    let mut right = chunk_heights(&terrain, IVec2::new(1, 0), 1);
    assert!(digs.apply_to_grid(&terrain, IVec2::new(0, 0), planet, &mut left));
    assert!(digs.apply_to_grid(&terrain, IVec2::new(1, 0), planet, &mut right));
    let last = GRID_POINTS - 1;
    for j in 0..=last {
        assert!(
            (left[last][j] - right[0][j]).abs() < 1e-6,
            "edge mismatch at j={j}"
        );
    }
    assert!(digs.depth_at(planet, edge_dir, globe.radius()) > 1.4);
    assert!(digs.depth_at(planet, edge_dir, globe.radius()) <= MAX_DIG_DEPTH);
}

/// Digs depress cumulative heights up to `MAX_DIG_DEPTH` and darken excavated facets on the mesh.
#[test]
fn digs_cap_depth_and_darken_mesh_facets() {
    let universe = Universe::new(19);
    let planet = universe.home();
    let globe = Arc::new(Globe::of(&universe.planet(planet)));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let coord = IVec2::ZERO;
    let span = Span::leaf(&terrain, coord, GRID_POINTS - 1);
    let dir = span.dir(16, 16);
    let mut digs = Digs::default();
    for _ in 0..40 {
        digs.push(DigStamp {
            planet,
            dir: dir.to_array(),
            radius: 2.0,
            depth: 1.0,
        });
    }
    assert!((digs.depth_at(planet, dir, terrain.globe().radius()) - MAX_DIG_DEPTH).abs() < 1e-4);
    let base = chunk_heights(&terrain, coord, 1);
    let mut dug = base.clone();
    assert!(digs.apply_to_grid(&terrain, coord, planet, &mut dug));
    let mut mesh_clean = chunk_mesh(&terrain, coord, &base, Lod::Near);
    let mut mesh_dug = chunk_mesh(&terrain, coord, &dug, Lod::Near);
    let frame = Frame::at(&terrain, span.center());
    digs.tint_mesh(&mut mesh_dug, planet, &frame, terrain.globe().radius());
    let _ = &mut mesh_clean;
    let bevy::mesh::VertexAttributeValues::Float32x2(uv_clean) =
        mesh_clean.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
    else {
        panic!();
    };
    let bevy::mesh::VertexAttributeValues::Float32x2(uv_dug) =
        mesh_dug.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
    else {
        panic!();
    };
    assert!(uv_clean != uv_dug);
}
