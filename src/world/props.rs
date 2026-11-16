//! The shared world material, the merged big-prop collider and ground-cover mesh, and spawning of a chunk's covers.

use avian3d::prelude::*;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::colormap;
use super::propmesh::{BIG, FLOWER, Plant};
use super::scatter::{Placement, Scatter};
use super::shapes::{FlatMesh, shade};
use super::solids::{Fan, fan};

/// One group of blades: fan shape, lift above the ground and sRGB color.
type Layer = (Fan, f32, [f32; 3]);

/// Layers of grass, reed, jungle leaf, fern and mushroom, in [`Plant`] order.
#[rustfmt::skip]
const COVER: [&[Layer]; FLOWER - BIG] = [
    &[([3.0, 0.12, 0.55, 0.0, 0.16], 0.0, [0.30, 0.58, 0.22])],
    &[([4.0, 0.1, 1.3, 0.0, 0.09], 0.0, [0.45, 0.6, 0.3]), ([3.0, 0.02, 0.25, 0.0, 0.08], 1.3, [0.45, 0.3, 0.15])],
    &[([5.0, 0.7, 0.2, 0.55, 0.5], 0.0, [0.12, 0.5, 0.2])],
    &[([5.0, 0.45, 0.2, 0.35, 0.22], 0.0, [0.2, 0.5, 0.2])],
    &[([2.0, 0.0, 0.18, 0.0, 0.07], 0.0, [0.9, 0.85, 0.75]), ([5.0, 0.11, -0.04, 0.0, 0.12], 0.18, [0.8, 0.12, 0.1])],
];

/// Petal and center colors of the flowers, in [`Plant`] order.
#[rustfmt::skip]
const HEADS: [([f32; 3], [f32; 3]); 6] = [
    ([0.88, 0.12, 0.1], [0.1, 0.07, 0.07]),
    ([0.97, 0.97, 0.95], [0.97, 0.8, 0.15]),
    ([0.98, 0.82, 0.12], [0.9, 0.55, 0.1]),
    ([0.35, 0.45, 0.95], [0.9, 0.9, 0.5]),
    ([0.97, 0.55, 0.7], [0.98, 0.85, 0.3]),
    ([0.9, 0.25, 0.7], [1.0, 0.75, 0.2]),
];

/// Collider radius and height of every big plant, in [`Plant`] order; zero means no collider.
#[rustfmt::skip]
const BODIES: [[f32; 2]; BIG] = [
    [0.3, 2.0], [0.3, 2.0], [0.3, 2.0], [0.3, 2.0], [0.25, 3.0], [0.35, 2.8], [0.45, 0.8],
    [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.7, 0.9], [0.7, 0.9], [0.7, 0.9],
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

/// Returns the trunk, cactus and rock colliders of every big prop as one compound, in chunk space.
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

/// Returns the blade groups of a ground-cover plant.
fn layers(plant: Plant) -> Vec<Layer> {
    let i = plant as usize - BIG;
    if let Some(layers) = COVER.get(i) {
        return layers.to_vec();
    }
    let (petal, heart) = HEADS[i - COVER.len()];
    vec![
        ([2.0, 0.0, 0.4, 0.0, 0.05], 0.0, [0.25, 0.5, 0.2]),
        ([4.0, 0.1, 0.0, 0.0, 0.1], 0.4, petal),
        ([3.0, 0.03, 0.02, 0.0, 0.06], 0.41, heart),
    ]
}

/// Builds the merged mesh of every small plant of a scatter: grass, flowers, ferns, mushrooms and reeds.
pub fn build_grass(scatter: &Scatter) -> Mesh {
    let mut m = FlatMesh::default();
    for (i, p) in scatter.grass.iter().enumerate() {
        let tone = 0.9 + (i % 5) as f32 * 0.05;
        for (spec, lift, color) in layers(p.plant) {
            let [count, reach, rise, arch, width] = spec;
            let s = p.scale;
            let shape = [count, reach * s, rise * s, arch * s, width * s];
            fan(
                &mut m,
                p.pos + Vec3::Y * lift * s,
                shape,
                p.yaw,
                shade(color, tone),
            );
        }
    }
    m.into_mesh()
}
