//! Infinite procedural archipelago: terrain, chunk streaming, props and sea.

mod biome;
mod chunks;
mod colormap;
mod focus;
mod islands;
mod mesh;
mod propmesh;
mod props;
mod scatter;
mod sea;
mod shapes;
mod solids;
pub mod structures;
mod surface;
mod terrain;

pub use biome::Biome;
pub use surface::Surface;
pub use terrain::Terrain;

use bevy::prelude::*;

use crate::saves::ActiveWorld;
use crate::screens::Screen;

/// Height of the sea surface in world units.
pub const SEA_LEVEL: f32 = 0.0;

/// Width of one square terrain chunk in world units.
pub const CHUNK_SIZE: f32 = 64.0;

/// Seed of the world to generate, inserted by the new game menu before `Screen::Loading`.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldSeed(pub u64);

/// Where the player spawns, inserted by the world when entering `Screen::Loading`.
///
/// A resumed world uses its saved body position.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct SpawnPoint(pub Vec3);

/// Whether the chunks around the spawn point have meshes and colliders.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldReady(pub bool);

/// Chunks stream in around the entity carrying this component.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct StreamAnchor;

/// Builds the terrain, spawn point and readiness flag from the world seed.
///
/// A resumed world spawns at its saved position instead of the terrain spawn.
fn begin_world(
    mut commands: Commands,
    seed: Option<Res<WorldSeed>>,
    active: Option<Res<ActiveWorld>>,
) {
    let terrain = Terrain::new(seed.map_or(0, |s| s.0));
    let saved = active.and_then(|active| active.0.save.position);
    let spawn = saved.map_or_else(|| terrain.find_spawn(), Vec3::from);
    commands.insert_resource(SpawnPoint(spawn));
    commands.insert_resource(terrain);
    commands.insert_resource(WorldReady(false));
}

/// Drops the world resources and pending chunk work.
fn end_world(mut commands: Commands, mut chunks: ResMut<chunks::Chunks>) {
    chunks.clear();
    commands.remove_resource::<Terrain>();
    commands.remove_resource::<SpawnPoint>();
    commands.remove_resource::<WorldReady>();
}

/// Run condition: the world exists and the player is loading or playing.
fn streaming(screen: Res<State<Screen>>, terrain: Option<Res<Terrain>>) -> bool {
    terrain.is_some() && matches!(screen.get(), Screen::Loading | Screen::Gameplay)
}

/// Registers the world systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((sea::plugin, structures::plugin))
        .init_resource::<chunks::Chunks>()
        .add_systems(Startup, chunks::setup_assets)
        .add_systems(
            OnEnter(Screen::Loading),
            (begin_world, sea::spawn_sea).chain(),
        )
        .add_systems(OnEnter(Screen::Title), end_world)
        .add_systems(
            Update,
            (
                chunks::add_instances.run_if(crate::render::ray_tracing::active),
                chunks::retarget.run_if(resource_changed::<crate::render::RayTracing>),
            ),
        )
        .add_systems(
            Update,
            (
                chunks::rebuild.run_if(resource_changed::<crate::render::RayTracing>),
                chunks::stream,
                chunks::collect,
                chunks::mark_ready,
                sea::follow_focus,
            )
                .chain()
                .run_if(streaming),
        );
}
