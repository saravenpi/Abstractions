//! Tests of the tolerance of the cloud shell mesh: it grows with the distance of the camera
//! from the shell, the mesh error stays under it at every height, and a high camera gets a
//! mesh many times smaller.

use std::f32::consts::PI;

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use super::mesh::wanted;
use super::shell::{Cap, TOLERANCE, build_within, cells_within, error};

/// Radius in metres of the planet the tests stand on.
const GROUND: f32 = 6.371e6;

/// Radius in metres of the shell halfway up its clouds.
const SHELL: f32 = 6.376e6;

/// Heights of the camera in metres: on the ground, high in the air and in orbit.
const HEIGHTS: [f32; 3] = [2.0, 1.0e4, 1.0e6];

/// Returns the number of triangles of `mesh` and the most metres any of its vertices lies from
/// the sphere of `radius` metres.
fn measure(mesh: &Mesh, radius: f32) -> (usize, f32) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions missing");
    };
    let Some(Indices::U32(indices)) = mesh.indices() else {
        panic!("indices missing");
    };
    let off = positions
        .iter()
        .map(|at| (Vec3::from(*at).length() - 1.0).abs() * radius);
    (indices.len() / 3, off.fold(0.0, f32::max))
}

/// The tolerance never drops under the finest one, and rises with the distance to the shell
/// by a hundredth of it.
#[test]
fn the_tolerance_grows_with_the_distance() {
    assert_eq!(wanted(GROUND, 2.0, SHELL), TOLERANCE);
    let tolerances: Vec<f32> = HEIGHTS.iter().map(|h| wanted(GROUND, *h, SHELL)).collect();
    assert!(
        tolerances.windows(2).all(|pair| pair[0] <= pair[1]),
        "{tolerances:?}"
    );
    let far = wanted(GROUND, 1.0e6, SHELL);
    assert!((far - 0.01 * (1.0e6 - 5.0e3)).abs() < 1.0, "{far}");
}

/// At 2 m, 10 km and 1,000 km, a mesh built for the tolerance the camera wants stays within it,
/// and the mesh from orbit has at least five times fewer triangles than the finest one.
#[test]
fn the_error_stays_under_the_tolerance_at_every_height() {
    let whole = Cap {
        centre: Vec3::Y,
        half: PI,
    };
    let finest = measure(&build_within(SHELL, whole, TOLERANCE), SHELL).0;
    for height in HEIGHTS {
        let tolerance = wanted(GROUND, height, SHELL);
        let (triangles, off) = measure(&build_within(SHELL, whole, tolerance), SHELL);
        assert!(off < tolerance, "{height}: {off} m off, wanted {tolerance}");
        assert!(
            error(SHELL, cells_within(SHELL, tolerance)) < tolerance,
            "{height}"
        );
        if height > 1.0e5 {
            assert!(triangles * 5 < finest, "{height}: {triangles} of {finest}");
        }
    }
}
