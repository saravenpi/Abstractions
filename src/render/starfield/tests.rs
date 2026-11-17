//! Starfield tests: the mesh faces the camera and the stars come from the real universe.

use bevy::math::DVec3;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

use super::{Speck, gather, mesh, star_speck};
use crate::universe::{GLY, Universe};

/// The starfield mesh has one inward-facing quad per speck and stays out of the ray tracing scene.
#[test]
fn starfield_faces_the_camera() {
    let dirs = [Vec3::X, Vec3::NEG_Y, Vec3::new(0.6, 0.0, -0.8)];
    let speck = |dir| Speck {
        dir,
        size: 0.01,
        color: Vec3::ONE,
        soft: 1.0,
    };
    let built = mesh(&dirs.map(speck));
    let Some(VertexAttributeValues::Float32x3(points)) = built.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions missing");
    };
    let Some(Indices::U32(indices)) = built.indices() else {
        panic!("indices missing");
    };
    assert_eq!((points.len(), indices.len()), (12, 18));
    assert!(!built.enable_raytracing);
    for corner in indices.chunks(3) {
        let [a, b, c] = [0, 1, 2].map(|i| Vec3::from(points[corner[i] as usize]));
        let inward = (b - a)
            .cross(c - a)
            .normalize()
            .dot(-dirs[corner[0] as usize / 4]);
        assert!(inward > 0.999, "{corner:?}");
    }
}

/// Nearer stars are brighter, and the stars gathered around the home star leave out that star itself.
#[test]
fn stars_come_from_the_universe() {
    let universe = Universe::new(11);
    let star = universe.system(universe.home().star).star;
    let near = star_speck(&star, star.pos + DVec3::X * 2.0 * GLY);
    let far = star_speck(&star, star.pos + DVec3::X * 30.0 * GLY);
    assert!((near.dir - Vec3::NEG_X).length() < 1e-5);
    assert!(near.size > far.size && near.color.length() > far.color.length());
    let seen = gather(&universe, star.id.galaxy, star.pos, star.pos);
    let others = universe.stars_near(star.id.galaxy, star.pos, 40.0).len() - 1;
    assert!(seen.len() >= others.min(4000));
    assert!(
        seen.iter()
            .all(|speck| speck.dir.is_normalized() && speck.size > 0.0)
    );
}
