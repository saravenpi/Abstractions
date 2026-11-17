//! The fixed particle pool: shared meshes and materials, and the entities that wear them.

use bevy::light::NotShadowCaster;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::kind::Kind;
use crate::screens::Screen;
use crate::space::Rebased;

/// Number of particle entities, created once per world.
pub const POOL_SIZE: usize = 160;

/// Half the width of the particle box around the camera, in meters. The box stands on the local
/// vertical of the camera.
pub const HALF_WIDTH: f32 = 14.0;

/// Depth of the particle box below the camera, in meters.
pub const BELOW: f32 = 5.0;

/// Height of the particle box above the camera, in meters.
pub const ABOVE: f32 = 9.0;

/// Colors per kind as linear-ish sRGB triples, and whether the material is unlit.
const LOOKS: [(Kind, [f32; 3], bool); 18] = [
    (Kind::Hidden, [0.0, 0.0, 0.0], false),
    (Kind::Leaf, [0.25, 0.5, 0.14], false),
    (Kind::Leaf, [0.35, 0.58, 0.18], false),
    (Kind::Leaf, [0.78, 0.68, 0.2], false),
    (Kind::Leaf, [0.8, 0.4, 0.12], false),
    (Kind::BigLeaf, [0.15, 0.45, 0.15], false),
    (Kind::BigLeaf, [0.2, 0.55, 0.2], false),
    (Kind::Petal, [0.95, 0.6, 0.72], false),
    (Kind::Petal, [0.98, 0.75, 0.82], false),
    (Kind::Snow, [0.95, 0.97, 1.0], false),
    (Kind::Dust, [0.8, 0.68, 0.45], false),
    (Kind::Dust, [0.7, 0.58, 0.38], false),
    (Kind::Spray, [0.9, 0.95, 1.0], false),
    (Kind::Bubble, [0.7, 0.9, 1.0], false),
    (Kind::Plankton, [0.5, 0.8, 0.7], true),
    (Kind::Plankton, [0.4, 0.7, 0.8], true),
    (Kind::Pollen, [1.0, 0.92, 0.5], false),
    (Kind::Pollen, [0.95, 0.85, 0.4], false),
];

/// One pooled particle.
#[derive(Component, Debug, Clone, Copy)]
pub struct Particle {
    /// What it is right now.
    pub kind: Kind,
    /// Seconds since it respawned.
    pub age: f32,
    /// Seconds it lives for.
    pub life: f32,
    /// Personal phase for flutter, tumble and blink.
    pub phase: f32,
    /// Height above sea level below which it respawns.
    pub floor: f32,
    /// False until its first placement, which may land anywhere in the box.
    pub primed: bool,
}

/// Shared meshes and materials for every particle.
#[derive(Resource, Debug, Clone)]
pub struct Pool {
    /// Square mesh for dots.
    pub quad: Handle<Mesh>,
    /// Diamond mesh for leaves and petals.
    pub diamond: Handle<Mesh>,
    /// Material for each look.
    pub looks: Vec<(Kind, Handle<StandardMaterial>)>,
}

impl Pool {
    /// Returns the material of the `n`th look of `kind`, wrapping around.
    pub fn material(&self, kind: Kind, n: usize) -> &Handle<StandardMaterial> {
        let count = self.looks.iter().filter(|(k, _)| *k == kind).count().max(1);
        let (_, handle) = self
            .looks
            .iter()
            .filter(|(k, _)| *k == kind)
            .nth(n % count)
            .unwrap_or(&self.looks[0]);
        handle
    }

    /// Returns the mesh a kind is drawn with.
    pub fn mesh(&self, kind: Kind) -> &Handle<Mesh> {
        if kind.is_leafy() {
            &self.diamond
        } else {
            &self.quad
        }
    }
}

/// Builds the shared meshes and materials once at startup.
pub fn build(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let looks = LOOKS
        .iter()
        .map(|&(kind, [r, g, b], unlit)| {
            let material = StandardMaterial {
                base_color: Color::srgb(r, g, b),
                perceptual_roughness: 1.0,
                double_sided: true,
                cull_mode: None,
                unlit,
                opaque_render_method: OpaqueRendererMethod::Forward,
                ..default()
            };
            (kind, materials.add(material))
        })
        .collect();
    commands.insert_resource(Pool {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        diamond: meshes.add(RegularPolygon::new(0.5, 4)),
        looks,
    });
}

/// Spawns the particles when gameplay starts, unless they already exist.
///
/// They are free entities in render space, so each is marked to follow anchor rebases.
pub fn spawn(mut commands: Commands, pool: Res<Pool>, existing: Query<(), With<Particle>>) {
    if !existing.is_empty() {
        return;
    }
    for _ in 0..POOL_SIZE {
        commands.spawn((
            Particle {
                kind: Kind::Hidden,
                age: 0.0,
                life: 0.0,
                phase: 0.0,
                floor: 0.0,
                primed: false,
            },
            Mesh3d(pool.quad.clone()),
            MeshMaterial3d(pool.looks[0].1.clone()),
            Transform::from_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}
