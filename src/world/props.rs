//! The shared world material, the merged big-prop collider and spawning of a chunk's covers.

use avian3d::prelude::*;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::colormap;
use super::ground::GroundMaterial;
use super::propmesh::{BIG, merge_parts_on};
use super::scatter::{Placement, Rig, Scatter};
use super::shapes::FlatMesh;

mod cover;
mod kelp;
mod pond;
#[cfg(test)]
mod tests;

pub use cover::small_tris;

use cover::{solids, sprout};

/// Collider radius and height of every big plant, in `Plant` order; zero means no collider.
#[rustfmt::skip]
const BODIES: [[f32; 2]; BIG] = [
    [0.3, 2.0], [0.3, 2.0], [0.3, 2.0], [0.3, 2.0], [0.25, 3.0], [0.25, 2.4], [0.5, 3.0], [0.25, 2.8],
    [0.35, 3.0], [0.42, 3.0],
    [0.35, 2.8], [0.45, 0.8], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
    [0.7, 0.9], [0.7, 0.9], [0.7, 0.9],
    [0.0, 0.0], [1.3, 1.8], [0.0, 0.0], [1.1, 0.8], [0.9, 0.5], [0.3, 1.0], [0.5, 0.55],
];

/// The one material every world mesh shares.
#[derive(Resource, Clone)]
pub struct PropAssets {
    /// Material colored by the lookup texture, used by terrain, props and grass.
    pub material: Handle<GroundMaterial>,
}

impl PropAssets {
    /// Builds the material around the color lookup texture.
    pub fn new(materials: &mut Assets<GroundMaterial>, images: &mut Assets<Image>) -> Self {
        let base = StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(images.add(colormap::image())),
            perceptual_roughness: 0.95,
            reflectance: 0.05,
            ..default()
        };
        let material = materials.add(GroundMaterial {
            base,
            extension: default(),
        });
        Self { material }
    }
}

/// The merged meshes a chunk shows on top of its terrain.
pub struct Cover {
    /// Every big prop of the chunk in one mesh.
    pub props: Option<Handle<Mesh>>,
    /// Every small plant of the chunk in one mesh.
    pub grass: Option<Handle<Mesh>>,
}

/// Builds the merged mesh of every small plant of a scatter in chunk space: grass, flowers, kelp and small solids.
#[cfg(test)]
pub fn build_grass(scatter: &Scatter) -> Mesh {
    build_grass_on(scatter, &Rig::PLAIN)
}

/// Builds the merged mesh of every small plant of a scatter in the frame of `rig`.
///
/// Each plant is one rigid piece that stands along the local up at its origin: blades keep their true size,
/// kelp rises its true height and pond plants reach the sea level sphere from the pond floor. It reads only
/// its arguments and is safe on any thread.
pub fn build_grass_on(scatter: &Scatter, rig: &Rig) -> Mesh {
    let loose = scatter
        .grass
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.plant.solid());
    let parts = loose.map(|(i, p)| {
        let mut m = FlatMesh::default();
        sprout(&mut m, i, p);
        (p.pos, m)
    });
    merge_parts_on(parts, &solids(scatter), rig)
}

/// Returns the trunk, cactus, rock and wreck colliders of every big prop as one compound, in chunk space.
#[cfg(test)]
pub fn body_collider(props: &[Placement]) -> Option<Collider> {
    body_collider_on(props, &Rig::PLAIN)
}

/// Returns the trunk, cactus, rock and wreck colliders of every big prop as one compound, in the frame of `rig`.
///
/// Each cylinder keeps its true size and stands along the local up at the origin of its prop. It reads only
/// its arguments and is safe on any thread.
pub fn body_collider_on(props: &[Placement], rig: &Rig) -> Option<Collider> {
    let shapes: Vec<_> = props
        .iter()
        .filter_map(|p| {
            let [radius, height] = BODIES[p.plant as usize];
            let (radius, height) = (radius * p.scale, height * p.scale);
            (radius > 0.0).then(|| {
                let (origin, turn) = rig.frame(p.pos);
                let at = origin + turn * Vec3::Y * (height * 0.5);
                let rot = turn * Quat::from_rotation_y(p.yaw);
                (at, rot, Collider::cylinder(radius, height))
            })
        })
        .collect();
    (!shapes.is_empty()).then(|| Collider::compound(shapes))
}

/// Spawns the merged props and the ground cover as children of the chunk and returns the props entity and
/// the ground cover entity.
pub fn spawn_props(
    commands: &mut Commands,
    chunk: Entity,
    material: &Handle<GroundMaterial>,
    cover: Cover,
) -> [Option<Entity>; 2] {
    let mut spawned = [None, None];
    commands.entity(chunk).with_children(|c| {
        let mat = || MeshMaterial3d(material.clone());
        if let Some(mesh) = cover.props {
            spawned[0] = Some(c.spawn((Mesh3d(mesh), mat(), Transform::default())).id());
        }
        if let Some(mesh) = cover.grass {
            let part = (Mesh3d(mesh), mat(), Transform::default(), NotShadowCaster);
            spawned[1] = Some(c.spawn(part).id());
        }
    });
    spawned
}
