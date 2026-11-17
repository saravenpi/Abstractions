//! Fills the audio scene from the world: which planet, how high, how wet, how windy.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::scene::{AudioScene, altitude_fade, cloud_depth};
use crate::render::{DayClock, GameCamera, daylight};
use crate::screens::Screen;
use crate::ship::ShipSet;
use crate::space::{Anchor, Frame, Location, Travel, altitude};
use crate::universe::{PlanetSpec, Universe};
use crate::wind::Wind;
use crate::world::Terrain;

/// Depth in metres of the head under the sea surface at which the sound is fully muffled.
const HEAD_DEPTH: f32 = 0.35;

/// Ground height in metres, above or below the sea, within which the shore is heard.
const SHORE_REACH: f32 = 7.0;

/// Height in metres above the sea at which the shore is no longer heard.
const SHORE_CEILING: f32 = 45.0;

/// The planet the listener is at, kept so that its description is not rebuilt every frame.
#[derive(Resource, Debug, Default)]
struct Here(Option<(Universe, PlanetSpec)>);

/// Where the player is, when a world exists.
#[derive(SystemParam)]
struct Place<'w> {
    /// Planet and way of travel.
    location: Option<Res<'w, Location>>,
    /// Frame everything is rendered in.
    anchor: Option<Res<'w, Anchor>>,
    /// The universe of the world being played.
    universe: Option<Res<'w, Universe>>,
}

/// Writes the planet, the frame, the cabin, the daylight and the wind into the scene.
fn sense_place(
    place: Place,
    clock: Res<DayClock>,
    wind: Res<Wind>,
    mut here: ResMut<Here>,
    mut scene: ResMut<AudioScene>,
) {
    let (Some(location), Some(anchor), Some(universe)) =
        (place.location, place.anchor, place.universe)
    else {
        return;
    };
    let known = |(of, spec): &(Universe, PlanetSpec)| *of == *universe && spec.id == location.body;
    if !here.0.as_ref().is_some_and(known) {
        here.0 = Some((*universe, universe.planet(location.body)));
    }
    scene.planet_seed = here
        .0
        .as_ref()
        .map_or(scene.planet_seed, |(_, spec)| spec.seed);
    scene.home = location.body == universe.home();
    scene.in_space = !matches!(anchor.frame, Frame::Surface(_));
    scene.cabin = f32::from(u8::from(location.travel == Travel::InShip));
    scene.daylight = daylight(clock.phase);
    scene.wind = wind.strength;
}

/// Returns how close the shoreline is to chart position `at`, `height` metres above the sea.
fn shore(terrain: &Terrain, at: Vec3, height: f32) -> f32 {
    let ground = terrain.height(at.x, at.z);
    let near = 1.0 - (ground.abs() / SHORE_REACH).clamp(0.0, 1.0);
    near * (1.0 - height / SHORE_CEILING).clamp(0.0, 1.0)
}

/// Writes the altitude fade, the water, the clouds and the shore into the scene from the camera.
///
/// In space there is no air, no water and no shore: all four are zero.
fn sense_listener(
    here: Res<Here>,
    anchor: Option<Res<Anchor>>,
    terrain: Option<Res<Terrain>>,
    camera: Query<&GlobalTransform, With<GameCamera>>,
    mut scene: ResMut<AudioScene>,
) {
    let (Some(anchor), Some(terrain), Some((_, spec))) = (anchor, terrain, here.0.as_ref()) else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };
    if !matches!(anchor.frame, Frame::Surface(_)) {
        (
            scene.altitude_fade,
            scene.submerged,
            scene.cloud_depth,
            scene.shore,
        ) = (0.0, 0.0, 0.0, 0.0);
        return;
    }
    let pos = camera.translation();
    let height = altitude(&anchor, terrain.globe(), pos);
    let chart = terrain.locate(anchor.to_frame(pos));
    scene.altitude_fade = altitude_fade(height, spec.atmosphere_height as f32);
    scene.submerged = (-height / HEAD_DEPTH).clamp(0.0, 1.0);
    scene.cloud_depth = cloud_depth(height, spec.cloud_base as f32, spec.cloud_top as f32);
    scene.shore = shore(&terrain, chart, height);
}

/// Puts the scene back to the title screen.
fn rest(mut scene: ResMut<AudioScene>) {
    *scene = AudioScene::default();
}

/// Registers the sensing systems, which run while a world is on screen, once the ship has put
/// the anchor where it is this frame.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Here>()
        .add_systems(OnExit(Screen::Gameplay), rest)
        .add_systems(
            Update,
            (sense_place, sense_listener)
                .chain()
                .in_set(super::Sensing)
                .after(ShipSet::Place)
                .run_if(in_state(Screen::Gameplay)),
        );
}
