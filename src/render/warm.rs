//! Pipeline warm-up: while the boot screen covers the loaded world, every material and mesh layout
//! in the world is drawn once, tiny, in front of the camera, so that no shader pipeline compiles
//! the first time something appears in play.
//!
//! Each material is drawn twice: once whole and once halfway through a dithered fade, because
//! a fading mesh (see `world::chunks::dither`) selects another pipeline variant.
//!
//! Materials of type `StandardMaterial` are covered here. A module with its own material type
//! calls [`warm_material`] once; meshes that only appear later go into [`Warm::add`].

use std::collections::HashSet;

use bevy::asset::UntypedAssetId;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{NoFrustumCulling, VisibilityRange};
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexAttributeId;
use bevy::prelude::*;

use super::GameCamera;
use crate::screens::WarmUp;

/// Size in metres of the largest side of a warm-up copy.
const SPECK: f32 = 0.01;

/// Distance in front of the camera of the warm-up copies, in metres.
const AHEAD: f32 = 1.5;

/// A visibility range whose fade spans the place of the copies, so the dithered variant of each
/// pipeline is drawn.
fn halfway() -> VisibilityRange {
    VisibilityRange {
        start_margin: 0.0..AHEAD * 2.0,
        end_margin: f32::MAX..f32::MAX,
        use_aabb: false,
    }
}

/// Frames the copies stay on screen: pipelines compile on other threads over several frames.
pub const WARM_FRAMES: u32 = 30;

/// Spawns a tiny instance of one material and mesh on the entity it is given.
type Spawner = Box<dyn Fn(&mut EntityCommands) + Send + Sync>;

/// Materials and meshes to draw during the warm-up although nothing in the world shows them yet,
/// such as effects spawned on demand. Any module adds its own with [`Warm::add`].
#[derive(Resource, Default)]
pub struct Warm {
    extra: Vec<Spawner>,
}

impl Warm {
    /// Draws `mesh` with `material` during every warm-up.
    pub fn add<M: Material>(&mut self, mesh: Handle<Mesh>, material: Handle<M>) {
        self.extra.push(Box::new(move |entity| {
            entity.insert((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));
        }));
    }
}

/// How far the warm-up has gone.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Warmed {
    /// Copies drawn in this warm-up.
    pub copies: usize,
    /// Frames the copies have been on screen.
    pub frames: u32,
    /// True once the copies stayed [`WARM_FRAMES`] frames.
    pub done: bool,
}

/// Marks the root of the warm-up copies.
#[derive(Component)]
struct Speck;

/// Ordering of the warm-up systems that run when the boot screen appears.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct Copies;

/// The material and the vertex layout of a mesh entity: what selects its pipeline. A mesh whose
/// data went to the render world has no readable layout and counts once per material.
type Key = (UntypedAssetId, Option<Vec<MeshVertexAttributeId>>);

/// Returns the scale that brings a mesh of bounds `aabb` down to [`SPECK`].
fn shrink(aabb: Option<&Aabb>) -> f32 {
    let size = aabb.map_or(1.0, |b| b.half_extents.max_element() * 2.0);
    SPECK / size.max(1.0e-4)
}

/// Copies every mesh entity drawn with a material of type `M` whose pipeline key is new, under
/// the warm-up root. Add it with [`warm_material`].
fn copy<M: Material>(
    mut commands: Commands,
    root: Single<Entity, With<Speck>>,
    meshes: Res<Assets<Mesh>>,
    drawn: Query<(&Mesh3d, &MeshMaterial3d<M>, Option<&Aabb>)>,
    mut warmed: ResMut<Warmed>,
) {
    let layout = |mesh: &Handle<Mesh>| {
        let attributes = meshes.get(mesh).and_then(|m| m.try_attributes().ok());
        attributes.map(|all| all.map(|(a, _)| a.id).collect())
    };
    let mut seen: HashSet<Key> = HashSet::new();
    for (mesh, material, aabb) in &drawn {
        if seen.insert((material.0.id().untyped(), layout(&mesh.0))) {
            let scale = Transform::from_scale(Vec3::splat(shrink(aabb)));
            let copy = (mesh.clone(), material.clone(), scale, NoFrustumCulling);
            commands.spawn((copy.clone(), ChildOf(*root)));
            commands.spawn((copy, halfway(), ChildOf(*root)));
        }
    }
    warmed.copies += seen.len();
}

/// Makes the warm-up draw every material of type `M` found in the world when the boot screen
/// appears. Call it once per material type, from the plugin that owns the type.
pub fn warm_material<M: Material>(app: &mut App) {
    app.add_systems(OnEnter(WarmUp::Hidden), copy::<M>.in_set(Copies));
}

/// Spawns the root of the copies in front of the camera, with every registered extra and one
/// shadowed light.
fn begin(mut commands: Commands, camera: Single<Entity, With<GameCamera>>, warm: Res<Warm>) {
    let pose = Transform::from_xyz(0.0, 0.0, -AHEAD);
    let root = commands
        .spawn((Speck, pose, Visibility::default(), ChildOf(*camera)))
        .id();
    for extra in &warm.extra {
        let speck = Transform::from_scale(Vec3::splat(SPECK));
        extra(&mut commands.spawn((speck, NoFrustumCulling, ChildOf(root))));
    }
    let light = PointLight {
        intensity: 1.0,
        range: 0.05,
        shadow_maps_enabled: true,
        ..default()
    };
    commands.spawn((light, NotShadowCaster, ChildOf(root)));
    commands.insert_resource(Warmed {
        copies: warm.extra.len(),
        ..default()
    });
}

/// Counts the frames the copies have been drawn.
fn count(mut warmed: ResMut<Warmed>) {
    warmed.frames += 1;
    if !warmed.done && warmed.frames >= WARM_FRAMES {
        warmed.done = true;
        info!("warm-up drew {} materials and mesh layouts", warmed.copies);
    }
}

/// Takes the copies away once the boot screen lifts.
fn clear(mut commands: Commands, specks: Query<Entity, With<Speck>>) {
    for speck in &specks {
        commands.entity(speck).despawn();
    }
}

/// Registers the warm-up and covers `StandardMaterial`.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Warm>()
        .init_resource::<Warmed>()
        .configure_sets(OnEnter(WarmUp::Hidden), Copies.after(begin))
        .add_systems(OnEnter(WarmUp::Hidden), begin)
        .add_systems(Update, count.run_if(in_state(WarmUp::Hidden)))
        .add_systems(OnExit(WarmUp::Hidden), clear);
    warm_material::<StandardMaterial>(app);
}
