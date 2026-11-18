//! Ball tests: the small sphere the sky draws keeps the colours of the far mesh of the world, class by
//! class and face by face, and carries the sea and its glow.

use std::sync::Arc;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::ball::{TRIANGLES, texel};
use super::{bake, ball};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::colormap;
use crate::world::globe::Globe;

/// Returns a 6 km globe of `class` whose sea covers the `ocean` share of it.
fn globe(class: PlanetClass, ocean: f64) -> Arc<Globe> {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    Arc::new(Globe::new(7, 6000.0, 9.8).with_record(record))
}

/// Returns the positions and the texture coordinates of `mesh`.
fn parts(mesh: &Mesh) -> (Vec<[f32; 3]>, Vec<[f32; 2]>) {
    let points = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(points)) => points.clone(),
        _ => Vec::new(),
    };
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uvs)) => uvs.clone(),
        _ => Vec::new(),
    };
    (points, uvs)
}

/// Returns the linear colour of every triangle of `mesh`: the colour of its first vertex, or without
/// vertex colours the texel of the colour lookup its texture coordinates name.
fn colours(mesh: &Mesh) -> Vec<Vec3> {
    if let Some(VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        let first = colours.iter().step_by(3);
        return first.map(|c| Vec3::new(c[0], c[1], c[2])).collect();
    }
    let image = colormap::image();
    let (_, uvs) = parts(mesh);
    uvs.iter().step_by(3).map(|uv| texel(&image, *uv)).collect()
}

/// Returns the mean linear colour of `mesh` over each of the six faces of the cube around it, every
/// triangle counted by the solid angle it covers, and after them the mean over the whole sphere.
fn faces(mesh: &Mesh) -> [Vec3; 7] {
    let (points, _) = parts(mesh);
    let (mut sums, mut areas) = ([Vec3::ZERO; 7], [0.0; 7]);
    for (corners, colour) in points.as_chunks::<3>().0.iter().zip(colours(mesh)) {
        let [a, b, c] = corners.map(|p| Vec3::from(p).normalize());
        let (area, mid) = ((b - a).cross(c - a).length(), (a + b + c).abs());
        let axis = mid.max_position();
        let face = 2 * axis + usize::from((a + b + c)[axis] < 0.0);
        for k in [face, 6] {
            sums[k] += colour * area;
            areas[k] += area;
        }
    }
    std::array::from_fn(|k| sums[k] / areas[k])
}

/// The ball is a unit sphere of 3,072 triangles with the relief of the planet and smooth normals.
#[test]
fn the_ball_is_a_small_unit_sphere() {
    let ball = ball(&globe(PlanetClass::Island, 0.6));
    let (points, uvs) = parts(&ball.mesh);
    assert_eq!(points.len(), TRIANGLES * 3);
    assert_eq!(points.len(), uvs.len());
    assert_eq!(colours(&ball.mesh).len(), TRIANGLES);
    let radii = points.iter().map(|p| Vec3::from(*p).length());
    let (low, high) = radii.fold((f32::MAX, 0.0_f32), |(l, h), r| (l.min(r), h.max(r)));
    assert!(low > 0.99 && high < 1.2 && high > low, "{low} to {high}");
}

/// On every class the ball has the colours of the far mesh the world draws just before the hand-over:
/// the same mean colour over the whole planet and over each face of the cube within 0.005, in linear
/// light.
#[test]
fn the_ball_keeps_the_colours_of_the_far_mesh() {
    let solid = |class: &PlanetClass| *class != PlanetClass::GasGiant;
    for class in PlanetClass::ALL.into_iter().filter(solid) {
        let globe = globe(class, 0.5);
        let ball = ball(&globe);
        let (near, far) = (faces(&bake(&globe)), faces(&ball.mesh));
        assert!(
            (ball.mean - far[6]).abs().max_element() < 0.005,
            "{class:?}"
        );
        for face in 0..7 {
            let off = (near[face] - far[face]).abs().max_element();
            assert!(off < 0.005, "{class:?} face {face}: off by {off}");
        }
    }
}

/// The ball knows its sea: none on a dry world, some on a wet one, and the light a sea of molten rock
/// gives off, which water does not.
#[test]
fn the_ball_carries_the_sea_and_its_glow() {
    let lava = ball(&globe(PlanetClass::Lava, 0.4));
    let look = PlanetClass::Lava.look();
    assert!(lava.sea > 0.01 && lava.sea < 0.9, "{}", lava.sea);
    assert_eq!(lava.glow, look.sea * look.glow);
    let dry = ball(&globe(PlanetClass::Lava, 0.0));
    assert_eq!((dry.sea, dry.glow), (0.0, Vec3::ZERO));
    let home = ball(&globe(PlanetClass::Island, 0.6));
    assert!(home.sea > 0.2 && home.sea < 0.95, "{}", home.sea);
    assert_eq!(home.glow, Vec3::ZERO);
    assert!(home.mean.z > home.mean.x, "{}", home.mean);
}

/// Prints what one ball costs: run with `-- --ignored --nocapture`.
#[test]
#[ignore = "a measurement, not a check"]
fn ball_cost() {
    for (class, radius) in [(PlanetClass::Island, 6000.0), (PlanetClass::Lava, 40_000.0)] {
        let record = PlanetRecord {
            class,
            ..PlanetRecord::TEMPERATE
        };
        let start = std::time::Instant::now();
        let globe = Arc::new(Globe::new(7, radius, 9.8).with_record(record));
        let built = start.elapsed();
        let ball = ball(&globe);
        let (points, uvs) = parts(&ball.mesh);
        let bytes = points.len() * (12 + 12 + 16 + 4) + uvs.len() * 8;
        println!(
            "{class:?} {radius} m: globe {built:?}, ball {:?} in all, {} triangles, {} kB",
            start.elapsed(),
            points.len() / 3,
            bytes / 1024
        );
    }
}
