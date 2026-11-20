//! Tests of the cloud shell mesh: it stays within the tolerance of the true sphere at every
//! level it is built at, for every size of planet, and it covers what the camera sees.

use std::f32::consts::PI;

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use super::shell::{Cap, TOLERANCE, build, cells, error, reach};

/// Radii in metres of planets from the smallest rocky one to the largest.
const RADII: [f32; 6] = [1.5e6, 3.0e6, 5.0e6, 6.371e6, 9.0e6, 1.2e7];

/// Returns the positions and the triangles of `mesh`.
fn triangles(mesh: &Mesh) -> (Vec<Vec3>, Vec<[usize; 3]>) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions missing");
    };
    let Some(Indices::U32(indices)) = mesh.indices() else {
        panic!("indices missing");
    };
    let at = positions.iter().map(|p| Vec3::from(*p)).collect();
    let corners = indices
        .chunks(3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    (at, corners)
}

/// Returns how many metres the flat face of a triangle lies from the sphere of `radius` metres at
/// the foot of the perpendicular from the middle of the sphere, where it sags the most.
fn sag(at: &[Vec3], corners: [usize; 3], radius: f32) -> f32 {
    let [a, b, c] = corners.map(|corner| at[corner]);
    let up = (b - a).cross(c - a).normalize();
    (up.dot(a).abs() - 1.0).abs() * radius
}

/// At every size of planet, no face of the whole shell lies farther than the tolerance from the
/// true sphere, and neither does any vertex.
#[test]
fn the_shell_stays_within_the_tolerance_of_the_sphere() {
    for radius in RADII {
        let whole = Cap {
            centre: Vec3::Y,
            half: PI,
        };
        let (at, corners) = triangles(&build(radius, whole));
        let worst = corners
            .iter()
            .map(|corner| sag(&at, *corner, radius))
            .fold(0.0, f32::max);
        assert!(worst < TOLERANCE + 1.0, "{radius}: faces {worst} m off");
        let high = at.iter().map(|p| (p.length() - 1.0).abs() * radius);
        assert!(high.fold(0.0, f32::max) < TOLERANCE, "{radius}");
        assert!(error(radius, cells(radius)) < TOLERANCE, "{radius}");
    }
}

/// The cells grow with the planet and are never fewer than the tolerance needs.
#[test]
fn bigger_planets_get_more_cells() {
    let counts: Vec<u32> = RADII.iter().map(|radius| cells(*radius)).collect();
    assert!(counts.windows(2).all(|pair| pair[0] <= pair[1]), "{counts:?}");
    for (radius, count) in RADII.iter().zip(&counts) {
        assert!(error(*radius, count - 8) >= TOLERANCE, "{radius}");
    }
}

/// Seen from the ground of Earth the shell is a few thousand triangles, and the mesh holds every
/// direction the camera can see: a vertex lies within a cell of each.
#[test]
fn the_mesh_covers_what_the_camera_sees_and_little_more() {
    let (ground, shell) = (6.371e6, 6.377e6);
    let need = reach(ground, 2.0, shell);
    assert!(need > 0.03 && need < 0.06, "{need}");
    let cap = Cap {
        centre: Vec3::Y,
        half: need * 1.5,
    };
    let (at, corners) = triangles(&build(shell, cap));
    assert!(corners.len() < 20_000, "{}", corners.len());
    let cell = std::f32::consts::FRAC_PI_2 / cells(shell) as f32;
    for turn in 0..16 {
        let (sin, cos) = (turn as f32 * PI / 8.0).sin_cos();
        let aside = Vec3::new(cos, 0.0, sin) * need.sin();
        let seen = (Vec3::Y * need.cos() + aside).normalize();
        let near = at.iter().map(|p| p.normalize().angle_between(seen));
        assert!(near.fold(f32::MAX, f32::min) < 2.0 * cell, "{turn}");
    }
}

/// From orbit the camera sees more of the shell than from the ground, up to about half of it
/// near the planet.
#[test]
fn the_reach_grows_with_the_altitude() {
    let (ground, shell) = (6.371e6, 6.377e6);
    let heights = [0.0, 1.0e4, 1.0e5, 1.0e6, 3.0e6];
    let reaches: Vec<f32> = heights.iter().map(|h| reach(ground, *h, shell)).collect();
    assert!(reaches.windows(2).all(|pair| pair[0] < pair[1]), "{reaches:?}");
    assert!(reaches[4] < PI / 2.0);
}
