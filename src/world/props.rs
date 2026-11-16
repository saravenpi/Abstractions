//! Shared prop meshes and the system of spawning a chunk's props as children.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::propmesh::{TRUNK_HEIGHT, bush_mesh, grass_mesh, rock_mesh, tree_mesh};
use super::scatter::{Placement, Scatter};

/// Prop meshes and the one material every world mesh shares.
#[derive(Resource, Clone)]
pub struct PropAssets {
    /// Vertex-colored material used by terrain, props and grass.
    pub material: Handle<StandardMaterial>,
    trees: [Handle<Mesh>; 2],
    rocks: [Handle<Mesh>; 2],
    bush: Handle<Mesh>,
}

impl PropAssets {
    /// Builds every shared prop mesh and the vertex-color material.
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.95,
            reflectance: 0.05,
            ..default()
        });
        Self {
            material,
            trees: [meshes.add(tree_mesh(0)), meshes.add(tree_mesh(1))],
            rocks: [meshes.add(rock_mesh(3)), meshes.add(rock_mesh(8))],
            bush: meshes.add(bush_mesh()),
        }
    }
}

/// Returns the transform of a placed prop, scaled by `size` times its own scale.
fn place(p: &Placement, size: f32) -> Transform {
    Transform::from_translation(p.pos)
        .with_rotation(Quat::from_rotation_y(p.yaw))
        .with_scale(Vec3::splat(p.scale * size))
}

/// Returns a compound collider holding one shape at a local offset.
fn offset(shape: Collider, y: f32) -> Collider {
    Collider::compound(vec![(Vec3::Y * y, Quat::IDENTITY, shape)])
}

/// Spawns trees, rocks, bushes and the merged grass mesh as children of the chunk entity.
pub fn spawn_props(
    commands: &mut Commands,
    chunk: Entity,
    assets: &PropAssets,
    scatter: &Scatter,
    grass: Handle<Mesh>,
) {
    let mat = || MeshMaterial3d(assets.material.clone());
    commands.entity(chunk).with_children(|c| {
        for (i, p) in scatter.trees.iter().enumerate() {
            let collider = offset(Collider::cylinder(0.3, TRUNK_HEIGHT * 2.0), TRUNK_HEIGHT);
            c.spawn((
                Mesh3d(assets.trees[i % 2].clone()),
                mat(),
                place(p, 1.0),
                collider,
            ));
        }
        for (i, p) in scatter.rocks.iter().enumerate() {
            let collider = offset(Collider::sphere(0.65), 0.4);
            c.spawn((
                Mesh3d(assets.rocks[i % 2].clone()),
                mat(),
                place(p, 1.4),
                collider,
            ));
        }
        for p in &scatter.bushes {
            c.spawn((Mesh3d(assets.bush.clone()), mat(), place(p, 1.0)));
        }
        if !scatter.grass.is_empty() {
            c.spawn((Mesh3d(grass), mat(), Transform::default()));
        }
    });
}

/// Builds the merged grass mesh for a scatter.
pub fn build_grass(scatter: &Scatter) -> Mesh {
    let tufts: Vec<(Vec3, f32)> = scatter.grass.iter().map(|p| (p.pos, p.yaw)).collect();
    grass_mesh(&tufts)
}
