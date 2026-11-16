//! Two thin flapping wings hinged on a body, shared by butterflies and birds.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

/// One wing, hinged at the body edge: `1.0` for the right wing, `-1.0` for the left.
#[derive(Component, Debug, Clone, Copy)]
pub struct Wing(pub f32);

/// Shape and look of a pair of wings.
pub struct Spec {
    /// Shared flat wing mesh.
    pub mesh: Handle<Mesh>,
    /// Wing material.
    pub material: Handle<StandardMaterial>,
    /// Length from the hinge to the tip in meters.
    pub length: f32,
    /// Width of the wing in meters.
    pub width: f32,
    /// Distance of the hinge from the body axis in meters.
    pub hinge: f32,
}

/// Returns one wing entity on side `side`.
fn wing(side: f32, spec: &Spec) -> impl Bundle {
    (
        Wing(side),
        Mesh3d(spec.mesh.clone()),
        MeshMaterial3d(spec.material.clone()),
        Transform::from_xyz(side * spec.hinge, 0.0, 0.0).with_scale(Vec3::new(
            side * spec.length,
            1.0,
            spec.width,
        )),
        NotShadowCaster,
    )
}

/// Returns the pair of wings as children of the entity it is spawned with.
pub fn pair(spec: &Spec) -> impl Bundle {
    children![wing(1.0, spec), wing(-1.0, spec)]
}

/// Rotates every wing among `children` to `angle` radians above the body plane.
pub fn pose(wings: &mut Query<(&mut Transform, &Wing)>, children: &Children, angle: f32) {
    let mut found = wings.iter_many_mut(children.iter());
    while let Some(Ok((mut transform, wing))) = found.fetch_next() {
        transform.rotation = Quat::from_rotation_z(wing.0 * angle);
    }
}
