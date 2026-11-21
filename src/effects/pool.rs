//! The fixed particle pool: shared meshes and materials, and the entities that wear them.

use bevy::light::NotShadowCaster;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::disc::soft_disc;
use super::kind::Kind;
use super::shed::SLOTS;
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

/// Colors per kind, for the kinds that do not take the colour of a plant, as linear-ish sRGB triples,
/// and whether the material is unlit.
const LOOKS: [(Kind, [f32; 3], bool); 11] = [
    (Kind::Hidden, [0.0, 0.0, 0.0], false),
    (Kind::Snow, [0.95, 0.97, 1.0], false),
    (Kind::Dust, [0.8, 0.68, 0.45], false),
    (Kind::Dust, [0.7, 0.58, 0.38], false),
    (Kind::Spray, [0.9, 0.95, 1.0], false),
    (Kind::Bubble, [0.7, 0.9, 1.0], false),
    (Kind::Plankton, [0.5, 0.8, 0.7], true),
    (Kind::Plankton, [0.4, 0.7, 0.8], true),
    (Kind::Pollen, [1.0, 0.92, 0.5], false),
    (Kind::Pollen, [0.95, 0.85, 0.4], false),
    (Kind::Rain, [0.72, 0.78, 0.88], false),
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
    /// Edge length in meters: the size of a leaf of its plant, or the size of its kind.
    pub size: f32,
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
    /// Soft round disc for the drops of spray and the bubbles: see [`soft_disc`].
    pub disc: Handle<Mesh>,
    /// Material for each look.
    pub looks: Vec<(Kind, Handle<StandardMaterial>)>,
    /// Leaf material of each slot of the plants nearby, in the colour of the plant of the slot.
    pub leaves: Vec<Handle<StandardMaterial>>,
    /// Petal material of each slot of the plants nearby.
    pub petals: Vec<Handle<StandardMaterial>>,
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

/// Returns the lit, double sided, forward rendered material of linear-ish sRGB `rgb`, `unlit` or not.
fn matte(rgb: [f32; 3], unlit: bool) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
        perceptual_roughness: 1.0,
        double_sided: true,
        cull_mode: None,
        unlit,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
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
        .map(|&(kind, rgb, unlit)| (kind, materials.add(matte(rgb, unlit))))
        .collect();
    let mut slots = || -> Vec<_> {
        (0..SLOTS)
            .map(|_| materials.add(matte([1.0; 3], false)))
            .collect()
    };
    let (leaves, petals) = (slots(), slots());
    commands.insert_resource(Pool {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        diamond: meshes.add(RegularPolygon::new(0.5, 4)),
        disc: meshes.add(soft_disc()),
        looks,
        leaves,
        petals,
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
                size: 0.0,
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
