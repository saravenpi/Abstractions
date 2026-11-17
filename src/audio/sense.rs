//! Fills the audio scene from the world: which planet, how high, how wet, how windy.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::scene::{AudioScene, altitude_fade, cloud_depth};
use super::shore::{closeness, nearest};
use crate::companion::Companion;
use crate::gear::{FlightFx, GearSet};
use crate::render::{DayClock, GameCamera, daylight, smoothstep};
use crate::screens::Screen;
use crate::ship::ShipSet;
use crate::space::{Anchor, Frame, Location, Travel, altitude};
use crate::universe::{PlanetSpec, Universe};
use crate::wind::Wind;
use crate::world::Terrain;

/// Depth in metres of the head under the sea surface at which the sound is fully muffled.
const HEAD_DEPTH: f32 = 0.35;

/// Distance in metres at which the voice of the companion has lost half its level.
const EARSHOT: f32 = 6.0;

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

/// Returns how close the nearest shoreline is to a listener at chart position `at`, `height`
/// metres above the sea, and the render position of that shoreline. Nothing is heard inland
/// or out at sea.
fn shore(terrain: &Terrain, anchor: &Anchor, at: Vec3, height: f32) -> (f32, Vec3) {
    let ground = |point: Vec2| terrain.height(point.x, point.y);
    let Some(offset) = nearest(ground, Vec2::new(at.x, at.z)) else {
        return (0.0, Vec3::ZERO);
    };
    let edge = terrain.embed(at.x + offset.x, 0.0, at.z + offset.y);
    (closeness(offset.length(), height), anchor.to_render(edge))
}

/// Returns the side a sound at render position `from` is on for the `listener`: -1 at its
/// left, 1 at its right, 0 ahead, behind or at the head itself.
fn side(listener: &GlobalTransform, from: Vec3) -> f32 {
    let toward = (from - listener.translation()).normalize_or_zero();
    toward.dot(listener.right().as_vec3())
}

/// Writes where the companion is for the listener, so its voice comes from where it floats.
fn sense_companion(
    camera: Query<&GlobalTransform, With<GameCamera>>,
    companion: Query<&GlobalTransform, With<Companion>>,
    mut scene: ResMut<AudioScene>,
) {
    let (Ok(camera), Ok(companion)) = (camera.single(), companion.single()) else {
        return;
    };
    let from = companion.translation();
    let far = from.distance(camera.translation());
    scene.companion = Vec2::new(side(camera, from), 1.0 / (1.0 + far / EARSHOT));
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
    let (level, edge) = shore(&terrain, &anchor, chart, height);
    (scene.shore, scene.shore_side) = (level, side(camera, edge) * level.signum());
}

/// Airspeeds in metres per second between which the rush of air grows from nothing to full.
const RUSH_SPEED: (f32, f32) = (6.0, 75.0);

/// Writes the flight gear into the scene: the rush of air by airspeed, the boosters by their
/// push and the flutter of the cloth. The gear publishes nothing while nobody is on foot.
fn sense_flight(flight: Option<Res<FlightFx>>, mut scene: ResMut<AudioScene>) {
    let fx = flight.as_deref().copied().unwrap_or_default();
    scene.rush = smoothstep(RUSH_SPEED.0, RUSH_SPEED.1, fx.airspeed);
    scene.boost = fx.thrust;
    scene.flutter = fx.flutter;
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
            (sense_place, sense_listener, sense_flight, sense_companion)
                .chain()
                .in_set(super::Sensing)
                .after(ShipSet::Place)
                .after(GearSet::Publish)
                .run_if(in_state(Screen::Gameplay)),
        );
}
