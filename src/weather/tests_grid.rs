//! Tests that the cover grid is laid out as the cube texture of the shaders is.

use bevy::math::DVec3;

use super::grid::{FACE, Patch, TEXELS, moment, texel, texel_dir};

/// Returns the texel `patch` lies on, which must be within a thousandth of a texel of its centre.
fn nearest(patch: &Patch) -> [i32; 2] {
    let at = [0, 1].map(|axis| patch.corner[axis] as f32 + patch.part[axis]);
    assert!(
        at.iter().all(|at| (at - at.round()).abs() < 1.0e-3),
        "{at:?}"
    );
    at.map(|at| at.round() as i32)
}

/// Every texel is its own nearest texel, in the order the texture counts them, and the faces
/// are in the order +X, -X, +Y, -Y, +Z, -Z with the columns and rows of a cube map.
#[test]
fn the_grid_is_laid_out_as_the_cube_texture() {
    let side = FACE as usize;
    for index in (0..TEXELS).step_by(37) {
        let patch = Patch::around(texel(index));
        let within = index % (side * side);
        assert_eq!(patch.face, index / (side * side));
        assert_eq!(
            nearest(&patch),
            [within % side, within / side].map(|at| at as i32)
        );
    }
    let majors = [
        DVec3::X,
        DVec3::NEG_X,
        DVec3::Y,
        DVec3::NEG_Y,
        DVec3::Z,
        DVec3::NEG_Z,
    ];
    for (face, major) in majors.into_iter().enumerate() {
        assert_eq!(Patch::around(major).face, face);
        assert!(texel_dir(face, FACE / 2, FACE / 2).dot(major) > 0.99);
    }
    assert!(texel_dir(0, 0, 0).z > 0.0 && texel_dir(0, 0, 0).y > 0.0);
    assert!(texel_dir(2, 0, 0).x < 0.0 && texel_dir(2, 0, 0).z < 0.0);
}

/// A texel one past an edge is the texel of the next face that touches the same stretch of
/// that edge, as the graphics card reads across the edges of a cube map, and the snapshots
/// are counted from second zero.
#[test]
fn a_texel_past_an_edge_is_the_neighbour_on_the_next_face() {
    for face in 0..6 {
        for row in [0, 17, FACE - 1] {
            let past = Patch::around(texel_dir(face, FACE, row));
            assert_ne!(past.face, face);
            assert!(nearest(&past).iter().any(|at| [0, FACE - 1].contains(at)));
            let gap = texel_dir(face, FACE, row).distance(texel_dir(face, FACE - 1, row));
            assert!(gap < 2.1 / f64::from(FACE), "{gap}");
        }
    }
    assert_eq!((moment(0.0), moment(24.0)), ((0, 0.0), (1, 0.5)));
    assert_eq!(moment(-4.0), (-1, 0.75));
}
