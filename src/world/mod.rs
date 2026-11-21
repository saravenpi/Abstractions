//! The planet under the player: terrain, quadtree chunk streaming, props and sea.

mod biome;
mod blades;
mod calendar;
pub mod canopy;
mod chunks;
mod colormap;
#[cfg(test)]
pub(crate) mod cpu;
mod enter;
mod focus;
mod footing;
pub mod globe;
mod ground;
mod islands;
mod lands;
mod mesh;
mod meter;
mod propmesh;
mod props;
mod reserve;
pub mod rocks;
mod scatter;
mod sea;
mod shapes;
mod solids;
pub mod structures;
mod surface;
mod terrain;
#[cfg(test)]
mod tests_enter;
#[cfg(test)]
mod tests_footing;

pub use biome::{Biome, ground_rgb};
pub use blades::{BladeAssets, BladeMaterial, Cropped, TurfUniform, turf_at};
pub use chunks::{Calm, Chunks, Reach, STEPS as RENDER_STEPS, Sited, dither, leaf_arc};
pub use enter::{Resettled, enter_planet};
pub use focus::{Link, spin};
pub use globe::Globe;
pub use ground::GroundMaterial;
pub use lands::Reading;
pub use meter::{Cost, Meter, STEPS as METER_STEPS};
pub use sea::{Liquid, wake};
pub use surface::Surface;
pub use terrain::Terrain;

use std::sync::Arc;

use bevy::prelude::*;

use enter::{arrive, follow_body};

use crate::home::Base;
use crate::saves::{ActiveWorld, WorldSave, unplayed};
use crate::screens::Screen;
use crate::space::{Anchor, Location, Travel};
use crate::universe::{BodyId, Universe};

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

/// Returns the view of the cube face that holds the spot `save` was taken at, rooted near it, or
/// `north` when there is no such spot.
fn saved_view(north: &Terrain, save: Option<&WorldSave>) -> Terrain {
    let globe = north.globe();
    let place = save.and_then(|save| save.place);
    let Some(at) = place.and_then(|place| place.position(globe.radius())) else {
        return north.clone();
    };
    Terrain::near(globe, at)
}

/// Returns the terrain view and the chart position a world starts at on the planet whose north face is
/// `north`: the spot `save` was taken at, or for a new game, which starts on the home planet, the spawn
/// the home base is looked for around, on the face that holds it. No ground near a pole is searched or
/// loaded behind the loading screen of a new game.
fn start(north: &Terrain, save: Option<&WorldSave>) -> (Terrain, Vec3) {
    if let Some(save) = save.filter(|save| save.place.is_some()) {
        let view = saved_view(north, Some(save));
        return chunks::rehome(&view, save.spawn(&view));
    }
    let globe = north.globe();
    let at = Base::landing(globe);
    let view = Terrain::near(globe, at);
    let spawn = view.locate(at);
    (view, spawn)
}

/// Builds the universe, the planet the world starts on, its terrain view, the spawn point, the
/// anchor and the readiness flag from the world seed.
///
/// A new world starts on the home planet where its base is looked for. A resumed world starts on the
/// planet and at the spot its save names, unless that is a body without ground, a body that does not
/// exist or a spot that points nowhere: then it starts as a new world. A spot at a height no body can be
/// at is put on the ground first. The terrain is the chart of the cube face that holds the spawn.
fn begin_world(
    mut commands: Commands,
    seed: Option<Res<WorldSeed>>,
    mut active: Option<ResMut<ActiveWorld>>,
) {
    let universe = Universe::new(seed.map_or(0, |s| s.0));
    let (body, north) = arrive(&mut commands, &universe, active.as_mut());
    let save = active.as_ref().map(|active| &active.0.save);
    let (terrain, spawn) = start(&north, save);
    commands.insert_resource(SpawnPoint(spawn));
    commands.insert_resource(Anchor::surface(body, &terrain, spawn));
    commands.insert_resource(Location {
        body,
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
    commands.remove_resource::<Resettled>();
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
    app.add_plugins((sea::plugin, structures::plugin, calendar::plugin))
        .init_resource::<chunks::Chunks>()
        .init_resource::<chunks::Budget>()
        .init_resource::<chunks::Calm>()
        .insert_resource(chunks::Reach::of(None))
        .init_resource::<Meter>()
        .init_resource::<rocks::Taken>()
        .add_message::<rocks::RockTaken>()
        .add_systems(Startup, chunks::setup_assets)
        .add_systems(
            OnEnter(Screen::Loading),
            (rocks::load, begin_world, sea::spawn_sea).chain(),
        )
        .add_systems(
            Update,
            rocks::store.run_if(resource_changed::<rocks::Taken>),
        )
        .add_systems(OnEnter(Screen::Title), end_world);
    app.add_plugins((ground::plugin, blades::plugin));
    stream_systems(app);
    app.add_systems(
        Update,
        (
            chunks::note.run_if(on_message::<rocks::RockTaken>),
            chunks::mend.run_if(chunks::mending),
        )
            .chain()
            .after(chunks::refine)
            .run_if(streaming),
    );
}

/// Registers the streaming chain: the planet and face under the focus, the quadtree, the detail
/// chunks, what is drawn, and the poses, in that order.
///
/// A new game is not ready before the home base is found and the frame after, in which the
/// player wakes there: the ground that must be solid is the ground of the base.
fn stream_systems(app: &mut App) {
    let based = resource_exists::<Base>.and_then(not(resource_added::<Base>));
    let ready = chunks::mark_ready.run_if(not(unplayed).or_else(based));
    let planet = (
        (meter::start, chunks::set_budget, chunks::follow_setting),
        follow_body,
        reserve::keep_base,
        chunks::change_planet.run_if(resource_exists_and_changed::<Planet>),
        meter::lap::<0>,
    );
    app.add_systems(
        Update,
        (
            planet.chain(),
            (chunks::follow_face, meter::lap::<1>).chain(),
            (chunks::watch_far, meter::lap::<2>).chain(),
            (chunks::grow.run_if(reserve::settled), meter::lap::<3>).chain(),
            (chunks::stream.run_if(reserve::settled), meter::lap::<4>).chain(),
            (chunks::collect, meter::lap::<5>).chain(),
            (chunks::refine, meter::lap::<6>).chain(),
            (chunks::show, chunks::watch_calm, meter::lap::<7>).chain(),
            (ready, chunks::place.run_if(chunks::due), meter::lap::<8>).chain(),
            (sea::follow_focus, meter::lap::<9>).chain(),
        )
            .chain()
            .run_if(streaming),
    );
}
