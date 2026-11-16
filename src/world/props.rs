//! The shared world material, the merged big-prop collider and spawning of a chunk's covers.

use avian3d::prelude::*;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::colormap;
use super::propmesh::BIG;
use super::scatter::Placement;

mod cover;
mod kelp;
mod pond;

pub use cover::{build_grass, small_tris};

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
    pub material: Handle<StandardMaterial>,
}

impl PropAssets {
    /// Builds the material around the color lookup texture.
    pub fn new(materials: &mut Assets<StandardMaterial>, images: &mut Assets<Image>) -> Self {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(images.add(colormap::image())),
            perceptual_roughness: 0.95,
            reflectance: 0.05,
            ..default()
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

/// Returns the trunk, cactus, rock and wreck colliders of every big prop as one compound, in chunk space.
pub fn body_collider(props: &[Placement]) -> Option<Collider> {
    let shapes: Vec<_> = props
        .iter()
        .filter_map(|p| {
            let [radius, height] = BODIES[p.plant as usize];
            let (radius, height) = (radius * p.scale, height * p.scale);
            (radius > 0.0).then(|| {
                let at = p.pos + Vec3::Y * height * 0.5;
                (
                    at,
                    Quat::from_rotation_y(p.yaw),
                    Collider::cylinder(radius, height),
                )
            })
        })
        .collect();
    (!shapes.is_empty()).then(|| Collider::compound(shapes))
}

/// Spawns the merged props and ground cover as children of the chunk and returns the props entity.
pub fn spawn_props(
    commands: &mut Commands,
    chunk: Entity,
    material: &Handle<StandardMaterial>,
    cover: Cover,
) -> Option<Entity> {
    let mut props = None;
    commands.entity(chunk).with_children(|c| {
        let mat = || MeshMaterial3d(material.clone());
        if let Some(mesh) = cover.props {
            props = Some(c.spawn((Mesh3d(mesh), mat(), Transform::default())).id());
        }
        if let Some(mesh) = cover.grass {
            c.spawn((Mesh3d(mesh), mat(), Transform::default(), NotShadowCaster));
        }
    });
    props
}
