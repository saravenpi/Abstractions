//! Detail tests: the sphere picked for a body keeps every facet of its rim under two game pixels
//! from a dot to a disc filling the screen, and the finest surface gives a texel to every game
//! pixel wherever the sky first draws a body.

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use super::bake::SHELL_UV;
use super::light::{DETAIL, detail};
use super::lod::{AHEAD, KEEP, level, need, pixels_per_radian};
use super::sphere::{EDGE, SEGMENTS, WIDEST, level as sphere_level, sphere};
use crate::world::globe::far::{LEAVE, LIMIT, SIDES, handover, sky_from};

/// Returns the widest angle in radians an edge of `mesh` spans on the unit sphere.
fn widest(mesh: &Mesh) -> f32 {
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return f32::MAX;
    };
    let Some(Indices::U32(indices)) = mesh.indices() else {
        return f32::MAX;
    };
    let dir = |i: u32| Vec3::from(points[i as usize]).normalize();
    let mut widest = 0.0_f32;
    for t in indices.as_chunks::<3>().0 {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            widest = widest.max(dir(a).angle_between(dir(b)));
        }
    }
    widest
}

/// From a body half a pixel wide to one whose radius covers 320 game pixels, the sphere picked
/// has no facet on its rim longer than two pixels, and a bigger body never gets a coarser sphere.
#[test]
fn the_rim_stays_round_at_every_size() {
    let widths: Vec<f32> = SEGMENTS
        .iter()
        .map(|k| widest(&sphere(*k, false)))
        .collect();
    let mut last = 0;
    for step in 1..=640 {
        let pixels = step as f32 * 0.5;
        let picked = sphere_level(pixels);
        assert!(picked >= last, "{pixels}");
        last = picked;
        let edge = widths[picked] * pixels;
        let capped = picked == SEGMENTS.len() - 1 && pixels * 1.5708 * WIDEST / 256.0 > EDGE;
        assert!(capped || edge <= EDGE, "{pixels} px: facets of {edge} px");
    }
}

/// The largest the sky ever draws a body, its ground at [`LEAVE`] radii under the narrowest field
/// of view of the settings, still fits the finest sphere.
#[test]
fn the_finest_sphere_serves_the_hand_over() {
    let pixels = pixels_per_radian(60f32.to_radians()) / LEAVE as f32;
    let finest = SEGMENTS[SEGMENTS.len() - 1] as f32;
    assert!(
        pixels * std::f32::consts::FRAC_PI_2 * WIDEST / finest <= EDGE,
        "{pixels}"
    );
    assert!(pixels > 200.0, "{pixels}");
}

/// The ground sphere and the cloud shell are unit spheres with smooth normals, the shell marked
/// for the cloud shader.
#[test]
fn spheres_are_smooth_unit_balls() {
    for shell in [false, true] {
        let mesh = sphere(8, shell);
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("no positions");
        };
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("no normals");
        };
        assert!(
            points
                .iter()
                .all(|p| (Vec3::from(*p).length() - 1.0).abs() < 1.0e-5)
        );
        assert_eq!(points, normals);
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("no texture coordinates");
        };
        assert!(uvs.iter().all(|uv| (*uv == SHELL_UV) == shell));
        assert_eq!(mesh.indices().map(|i| i.len() / 3), Some(12 * 64));
    }
}

/// Every facet faces out: its corners turn counter-clockwise seen from outside the ball.
#[test]
fn facets_face_out() {
    let mesh = sphere(4, false);
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("no positions");
    };
    let Some(Indices::U32(indices)) = mesh.indices() else {
        panic!("no indices");
    };
    for t in indices.as_chunks::<3>().0 {
        let [a, b, c] = t.map(|i| Vec3::from(points[i as usize]));
        assert!((b - a).cross(c - a).dot(a + b + c) > 0.0);
    }
}

/// At the default field of view the finest surface gives a texel or more to every game pixel of a
/// body at [`LEAVE`] radii, the nearest the sky draws it, and every level is asked for before the
/// body needs it and kept a while after.
#[test]
fn the_finest_surface_gives_a_texel_to_every_pixel() {
    let ppr = pixels_per_radian(75f32.to_radians());
    let at_leave = need(ppr, 1.0, LEAVE as f32);
    assert!(at_leave <= SIDES[SIDES.len() - 1] as f32, "{at_leave}");
    assert!(at_leave > SIDES[1] as f32, "{at_leave}");
    assert_eq!(level(at_leave, 1.0), SIDES.len() - 1);
    for distance in [1.5, 2.0, 5.0, 20.0, 200.0, 2000.0] {
        let needed = need(ppr, 1.0, distance);
        let asked = level(needed, AHEAD);
        assert!(
            SIDES[asked] as f32 >= needed || asked == SIDES.len() - 1,
            "{distance}"
        );
        assert!(level(needed, KEEP) >= asked, "{distance}");
    }
    assert_eq!(level(need(ppr, 1.0, 2000.0), AHEAD), 0);
}

/// A body is a dot in its mean colour up to about a pixel, shows all of its surface from three
/// pixels on, never less as it grows, and all of it wherever the sky draws it.
#[test]
fn a_body_shows_its_surface_from_the_hand_over_inward() {
    assert_eq!((detail(0.0), detail(DETAIL.0)), (0.0, 0.0));
    assert_eq!((detail(DETAIL.1), detail(1.0)), (1.0, 1.0));
    let steps: Vec<f32> = (0..=40).map(|k| detail(k as f32 * 0.0005)).collect();
    assert!(steps.windows(2).all(|pair| pair[1] >= pair[0]));
    for radius in [3000.0, 6000.0, 40_000.0, 150_000.0] {
        let at = handover(radius);
        assert_eq!(detail((radius / at) as f32), 1.0, "{radius}");
        assert!((radius / at - 1.0 / LIMIT).abs() < 1.0e-12);
        assert_eq!(detail((radius / sky_from(radius)) as f32), 1.0, "{radius}");
        assert!((sky_from(radius) - LEAVE * radius).abs() < 1.0e-6 * radius);
    }
}
