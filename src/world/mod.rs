//! The planet under the player: terrain, quadtree chunk streaming, props and sea.

mod biome;
mod chunks;
mod colormap;
mod focus;
pub mod globe;
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
pub use chunks::Sited;
pub use focus::{Link, spin};
pub use globe::Globe;
pub use surface::Surface;
pub use terrain::Terrain;

use std::sync::Arc;

use bevy::prelude::*;

use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::space::{Anchor, Location, Travel};
use crate::universe::{BodyId, Universe};
use globe::Face;

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

/// The planet the world streams: the one the player stands on or flies around.
#[derive(Resource, Clone)]
pub struct Planet {
    /// Which planet of the universe this is.
    pub body: BodyId,
    /// Its ground.
    pub globe: Arc<Globe>,
    /// Seconds for one turn around its +Y axis.
    pub day_seconds: f64,
}

/// Makes `body` the streamed planet: swaps the globe and the terrain view, and has every chunk
/// of the previous planet dropped.
///
/// The caller still owns the `Anchor` and `Location`. The terrain starts on the north face and
/// follows the focus from the next frame on.
pub fn enter_planet(commands: &mut Commands, universe: &Universe, body: BodyId) -> Terrain {
    let spec = universe.planet(body);
    let globe = Arc::new(Globe::of(&spec));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    commands.insert_resource(terrain.clone());
    commands.insert_resource(Planet {
        body,
        globe,
        day_seconds: spec.day_seconds,
    });
    terrain
}

/// Enters the planet named by `Location` whenever it is not the streamed one.
fn follow_body(
    mut commands: Commands,
    location: Option<Res<Location>>,
    universe: Option<Res<Universe>>,
    planet: Res<Planet>,
) {
    if let (Some(location), Some(universe)) = (location, universe)
        && location.body != planet.body
    {
        enter_planet(&mut commands, &universe, location.body);
    }
}

/// Builds the universe, the home planet, its terrain view, the spawn point, the anchor and the
/// readiness flag from the world seed.
///
/// The terrain is the chart of the cube face that holds the spawn. A resumed world spawns at its
/// saved position instead of the terrain spawn.
fn begin_world(
    mut commands: Commands,
    seed: Option<Res<WorldSeed>>,
    active: Option<Res<ActiveWorld>>,
) {
    let universe = Universe::new(seed.map_or(0, |s| s.0));
    let home = universe.home();
    let north = enter_planet(&mut commands, &universe, home);
    let saved = active.and_then(|active| active.0.save.position);
    let found = saved.map_or_else(|| north.find_spawn(), Vec3::from);
    let (terrain, spawn) = chunks::rehome(&north, found);
    commands.insert_resource(SpawnPoint(spawn));
    commands.insert_resource(Anchor::surface(home, &terrain, spawn));
    commands.insert_resource(Location {
        body: home,
        travel: Travel::OnFoot,
    });
    commands.insert_resource(universe);
    commands.insert_resource(terrain);
    commands.insert_resource(WorldReady(false));
}

/// Drops the world resources and pending chunk work.
fn end_world(mut commands: Commands, mut chunks: ResMut<chunks::Chunks>) {
    chunks.clear();
    commands.remove_resource::<Terrain>();
    commands.remove_resource::<Planet>();
    commands.remove_resource::<SpawnPoint>();
    commands.remove_resource::<WorldReady>();
    commands.remove_resource::<Universe>();
    commands.remove_resource::<Anchor>();
    commands.remove_resource::<Location>();
}

/// Run condition: the planet exists and the player is loading or playing.
fn streaming(
    screen: Res<State<Screen>>,
    terrain: Option<Res<Terrain>>,
    planet: Option<Res<Planet>>,
) -> bool {
    terrain.is_some()
        && planet.is_some()
        && matches!(screen.get(), Screen::Loading | Screen::Gameplay)
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
        );
    stream_systems(app);
}

/// Registers the streaming chain: the planet and face under the focus, the quadtree, the detail
/// chunks, what is drawn, and the poses, in that order.
fn stream_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            chunks::rebuild.run_if(resource_changed::<crate::render::RayTracing>),
            follow_body,
            chunks::change_planet.run_if(resource_exists_and_changed::<Planet>),
            chunks::follow_face,
            chunks::watch_far,
            chunks::grow,
            chunks::stream,
            chunks::collect,
            chunks::refine,
            chunks::show,
            chunks::mark_ready,
            chunks::place,
            sea::follow_focus,
        )
            .chain()
            .run_if(streaming),
    );
}
