//! The meshes of the moving parts that have no home in the hull module: the pedal, the gear
//! lever, the warp switch and the strap, each built around its hinge at the origin.

use bevy::prelude::*;

use super::tune::{PEDAL_LENGTH, STRAP, THROW_LENGTH};
use crate::ship::solid::Solid;

/// Builds a lever of `length` metres standing on its hinge: a thin shaft under a handle of
/// `half` metres either way across and `grip` metres high.
fn lever(length: f32, half: f32, grip: f32) -> Mesh {
    let mut lever = Solid::default();
    let shaft = length - grip;
    lever.slab(
        Vec3::new(-0.007, 0.0, -0.007),
        Vec3::new(0.007, shaft, 0.007),
    );
    lever.slab(
        Vec3::new(-half, shaft, -half * 0.7),
        Vec3::new(half, length, half * 0.7),
    );
    lever.into_mesh()
}

/// Builds the gear lever: a shaft under a wide handle.
pub fn gear_mesh() -> Mesh {
    lever(THROW_LENGTH.0, 0.03, 0.04)
}

/// Builds the warp switch: a short toggle.
pub fn switch_mesh() -> Mesh {
    lever(THROW_LENGTH.1, 0.014, 0.025)
}

/// Builds a rudder pedal: a plate under the boot that runs from its heel at the origin to
/// its toe along -Z.
pub fn pedal_mesh() -> Mesh {
    let mut pedal = Solid::default();
    pedal.slab(
        Vec3::new(-0.045, -0.012, -PEDAL_LENGTH),
        Vec3::new(0.045, 0.012, 0.0),
    );
    pedal.into_mesh()
}

/// Builds the loose strap, hanging from its hinge at the origin: a band and its buckle.
pub fn strap_mesh() -> Mesh {
    let mut strap = Solid::default();
    let length = STRAP.1;
    strap.slab(
        Vec3::new(-0.012, -length, -0.003),
        Vec3::new(0.012, 0.0, 0.003),
    );
    strap.slab(
        Vec3::new(-0.018, -length, -0.007),
        Vec3::new(0.018, -length + 0.03, 0.007),
    );
    strap.into_mesh()
}
