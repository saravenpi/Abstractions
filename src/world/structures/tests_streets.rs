//! Tests that village streets follow the terrain mesh on true spheres.

use bevy::math::DVec3;

use super::Terrain;
use super::globes::{RADII, above_mesh, faces, planet, villages};
use super::site::Site;
use super::village;
use crate::world::mesh::Frame;

/// Lowest and highest allowed height of a road vertex above the terrain mesh, in meters: the bounds of the
/// flat world, with 2 mm for the rounding of chart positions.
const HUG: (f32, f32) = (-0.002, 0.15);

/// Deepest a road triangle may lie under the terrain mesh at its middle, in meters. A flat quad whose corners
/// hug the mesh still dips under a sharp crease between them, on flat land as on a globe.
const SUNK: f32 = -0.25;

/// Returns how far the planet position `p` lies above the terrain mesh of the chart `face`, read on the chart.
fn lift(face: &Terrain, p: DVec3) -> f32 {
    let at = face.locate(p);
    at.y - face.mesh_height(at.x, at.z)
}

/// Asserts that every plaza, street and footpath vertex of the village `site` of the chart `face` hugs the
/// terrain mesh along the local up, on the chart and, for one vertex in 40, on the triangles of the chunk
/// mesh itself, and that no road triangle is buried.
fn check_roads(face: &Terrain, site: Site) {
    let view = face.around(site.origin.x, site.origin.y);
    let frame = Frame::at(&view, site.origin);
    let roads = village::ground((face, &view), site, &frame);
    let world: Vec<DVec3> = roads.iter().map(|p| planet(&frame, *p)).collect();
    for (i, p) in world.iter().enumerate() {
        let up = lift(face, *p);
        let hugs = (HUG.0..=HUG.1).contains(&up);
        assert!(hugs, "road vertex {up} m above the mesh, {site:?}");
        let real = if i % 40 == 0 {
            above_mesh(face, *p)
        } else {
            up
        };
        let agree = (real - up).abs() < 0.01;
        assert!(agree, "chart says {up} m, triangles {real} m, {site:?}");
    }
    let tris = world.as_chunks::<3>().0.iter();
    let middles = tris.map(|t| lift(face, (t[0] + t[1] + t[2]) / 3.0));
    let deepest = middles.fold(f32::MAX, f32::min);
    assert!(deepest >= SUNK, "a road triangle lies {deepest} m deep");
}

/// On globes of 3 and 40 km, over at least 6 villages each: every plaza, street and footpath vertex lies
/// 0 to 15 cm above the terrain mesh along the local up, and no road triangle is buried.
#[test]
fn streets_follow_the_mesh_on_globes() {
    for radius in RADII {
        let charts = faces(42, radius);
        let sites = charts
            .iter()
            .flat_map(|face| villages(face, 2).into_iter().map(move |s| (face, s)));
        let seen = sites.map(|(face, site)| check_roads(face, site)).count();
        assert!(seen >= 6, "{seen} villages on the {radius} m globe");
    }
}
