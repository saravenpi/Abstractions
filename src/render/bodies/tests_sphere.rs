//! Sphere tests: the sphere picked for a body covers its ball and stands out of it by under half a
//! pixel from a dot to a disc filling the screen, at a cost of a few thousand triangles.

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use super::bake::SHELL_UV;
use super::lod::{pixels_per_radian, rim_pixels};
use super::sphere::{BULGE, SEGMENTS, bulge, growth, level as sphere_level, sphere};
use crate::world::globe::far::LEAVE;

/// Returns the corners and the facets of `mesh`.
fn parts(mesh: &Mesh) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("no positions");
    };
    let Some(Indices::U32(indices)) = mesh.indices() else {
        panic!("no indices");
    };
    let corners = points.iter().map(|p| Vec3::from(*p)).collect();
    (corners, indices.as_chunks::<3>().0.to_vec())
}

/// Returns the distance from the centre of the nearest facet plane of `mesh`, and the farthest
/// corner.
fn hull(mesh: &Mesh) -> (f32, f32) {
    let (corners, facets) = parts(mesh);
    let plane = |t: &[u32; 3]| {
        let [a, b, c] = t.map(|i| corners[i as usize]);
        (b - a).cross(c - a).normalize().dot(a)
    };
    let nearest = facets.iter().map(plane).fold(f32::MAX, f32::min);
    let farthest = corners.iter().map(|c| c.length()).fold(0.0, f32::max);
    (nearest, farthest)
}

/// Every sphere covers the unit ball, each facet outside it, and stands out of it by no more than
/// [`bulge`] says, so the level rule can trust it.
#[test]
fn every_sphere_covers_its_ball() {
    for segments in SEGMENTS {
        let (nearest, farthest) = hull(&sphere(segments, false));
        assert!(
            nearest >= 1.0 - 1.0e-6,
            "{segments}: a facet {nearest} from the centre"
        );
        assert!(
            farthest - 1.0 <= bulge(segments),
            "{segments}: out by {}",
            farthest - 1.0
        );
    }
}

/// From a body half a pixel wide to one whose rim covers 320 game pixels, the sphere picked stands
/// out of the ball by under half a pixel, and a bigger body never gets a coarser sphere.
#[test]
fn the_sphere_hugs_the_ball_at_every_size() {
    let mut last = 0;
    for step in 1..=640 {
        let pixels = step as f32 * 0.5;
        let picked = sphere_level(pixels);
        assert!(picked >= last, "{pixels}");
        last = picked;
        let out = bulge(SEGMENTS[picked]) * pixels;
        assert!(out <= BULGE, "{pixels} px: out by {out} px");
    }
}

/// Wherever the sky draws a body, its ground at [`LEAVE`] radii or farther under the narrowest
/// field of view of the settings, the finest sphere still hugs it and costs 12,288 triangles,
/// from three radii on 3,072, from ten radii 768 and from a hundred 192: the coarse terrain of the
/// world it replaces is about 700,000.
#[test]
fn a_far_body_keeps_to_its_triangle_budget() {
    let ppr = pixels_per_radian(60f32.to_radians());
    let triangles = |radii: f32| {
        let segments = SEGMENTS[sphere_level(rim_pixels(ppr, 1.0 / radii))];
        12 * segments * segments
    };
    let nearest = rim_pixels(ppr, 1.0 / LEAVE as f32);
    let finest = SEGMENTS[SEGMENTS.len() - 1];
    assert!(
        nearest > 300.0 && bulge(finest) * nearest <= BULGE,
        "{nearest}"
    );
    assert_eq!(12 * finest * finest, 12_288);
    let costs = [3.0, 10.0, 100.0].map(triangles);
    assert!(
        costs[0] <= 3072 && costs[1] <= 768 && costs[2] <= 192,
        "{costs:?}"
    );
}

/// The ground sphere and the cloud shell have the smooth normals of the unit sphere, their
/// corners grown along them by one share, the shell marked for the cloud shader.
#[test]
fn spheres_are_smooth_grown_balls() {
    for shell in [false, true] {
        let mesh = sphere(8, shell);
        let (corners, facets) = parts(&mesh);
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("no normals");
        };
        let units: Vec<[f32; 3]> = normals.clone();
        let flat: Vec<u32> = facets.iter().flatten().copied().collect();
        let grown = growth(&units, &flat);
        for (corner, normal) in corners.iter().zip(normals) {
            assert!((Vec3::from(*normal).length() - 1.0).abs() < 1.0e-5);
            assert!((*corner - Vec3::from(*normal) * grown).length() < 1.0e-5);
        }
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("no texture coordinates");
        };
        assert!(uvs.iter().all(|uv| (*uv == SHELL_UV) == shell));
        assert_eq!(facets.len(), 12 * 64);
    }
}

/// Every facet faces out: its corners turn counter-clockwise seen from outside the ball.
#[test]
fn facets_face_out() {
    let (corners, facets) = parts(&sphere(4, false));
    for t in &facets {
        let [a, b, c] = t.map(|i| corners[i as usize]);
        assert!((b - a).cross(c - a).dot(a + b + c) > 0.0);
    }
}

/// The rim of a ball covers its share of the view in pixels from afar and more as it comes near,
/// where the rim stands out at a wider angle than the radius over the distance.
#[test]
fn the_rim_grows_as_a_ball_comes_near() {
    let ppr = pixels_per_radian(75f32.to_radians());
    assert!((rim_pixels(ppr, 0.001) - ppr * 0.001).abs() < 1.0e-3);
    let near = rim_pixels(ppr, 1.0 / 1.5);
    assert!((near - ppr * 2.0 / 5f32.sqrt()).abs() < 1.0e-2, "{near}");
    assert!(rim_pixels(ppr, 2.0).is_finite());
}
