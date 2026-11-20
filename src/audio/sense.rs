//! Fills the audio scene from the world: which planet, how high, how wet, how windy.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::planet::{Surf, loudness, surf, wind as heard};
use super::scene::{AudioScene, altitude_fade};
use super::shore::{CEILING, ShoreRead, closeness};
use crate::companion::Companion;
use crate::gear::{FlightFx, GearSet};
use crate::net::Remote;
use crate::player::above_sea;
use crate::render::{CloudCrossing, DayClock, GameCamera, daylight, smoothstep};
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
pub(super) struct Here(pub(super) Option<(Universe, PlanetSpec)>);

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

/// Writes the planet, the frame, the cabin, the daylight and the wind into the scene. The wind
/// is not heard on a planet without air and never dies down on a gas giant.
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
    let record = here.0.as_ref().map(|(_, spec)| spec.record);
    scene.class = record.map_or(scene.class, |record| record.class);
    scene.home = location.body == universe.home();
    scene.in_space = !matches!(anchor.frame, Frame::Surface(_));
    scene.cabin = f32::from(u8::from(location.travel == Travel::InShip));
    scene.daylight = daylight(clock.phase);
    scene.wind = record.map_or(wind.strength, |record| heard(&record, wind.strength));
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
    companion: Query<&GlobalTransform, (With<Companion>, Without<Remote>)>,
    mut scene: ResMut<AudioScene>,
) {
    let (Ok(camera), Ok(companion)) = (camera.single(), companion.single()) else {
        return;
    };
    let from = companion.translation();
    let far = from.distance(camera.translation());
    scene.companion = Vec2::new(side(camera, from), 1.0 / (1.0 + far / EARSHOT));
}

/// Writes the altitude fade, the water and the clouds into the scene from the camera.
///
/// In space there is no air and no water: both are zero. On a planet the whoosh of the clouds
/// plays inside a real cloud only, as loud as its air is. The sound is muffled under the liquid
/// of the globe and never in a dry basin.
fn sense_listener(
    here: Res<Here>,
    (anchor, terrain): (Option<Res<Anchor>>, Option<Res<Terrain>>),
    (camera, crossing): (
        Query<&GlobalTransform, With<GameCamera>>,
        Option<Res<CloudCrossing>>,
    ),
    mut scene: ResMut<AudioScene>,
) {
    let (Some(anchor), Some(terrain), Some((_, spec))) = (anchor, terrain, here.0.as_ref()) else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };
    if !matches!(anchor.frame, Frame::Surface(_)) {
        (scene.altitude_fade, scene.submerged, scene.cloud_depth) = (0.0, 0.0, 0.0);
        return;
    }
    let pos = camera.translation();
    let height = altitude(&anchor, terrain.globe(), pos);
    scene.altitude_fade = altitude_fade(height, spec.atmosphere_height as f32);
    let depth = crossing.map_or(0.0, |crossing| crossing.depth);
    scene.cloud_depth = depth * loudness(&spec.record);
    let afloat = above_sea(&anchor, terrain.globe(), pos);
    scene.submerged = (-afloat / HEAD_DEPTH).clamp(0.0, 1.0);
}

/// Writes the shore into the scene from the camera: how close the nearest shoreline is and on
/// which side it lies. Nothing is heard in space, out at sea or inland, and where the record says
/// no surf breaks.
fn sense_shore(
    here: Res<Here>,
    (anchor, terrain, time): (Option<Res<Anchor>>, Option<Res<Terrain>>, Res<Time>),
    camera: Query<&GlobalTransform, With<GameCamera>>,
    mut read: Local<ShoreRead>,
    mut scene: ResMut<AudioScene>,
) {
    let (Some(anchor), Some(terrain), Some((_, spec))) = (anchor, terrain, here.0.as_ref()) else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };
    let pos = camera.translation();
    let height = altitude(&anchor, terrain.globe(), pos);
    let surfing = matches!(anchor.frame, Frame::Surface(_)) && surf(&spec.record) != Surf::Silent;
    if !surfing || height >= CEILING {
        (scene.shore, scene.shore_side) = (0.0, 0.0);
        return;
    }
    let at = terrain.locate(anchor.to_frame(pos)).xz();
    let (level, edge) = read
        .read(spec.id, time.elapsed_secs(), &terrain, at)
        .map_or((0.0, Vec3::ZERO), |point| {
            let render = anchor.to_render(terrain.embed(point.x, 0.0, point.y));
            (closeness(point.distance(at), height), render)
        });
    (scene.shore, scene.shore_side) = (level, side(camera, edge) * level.signum());
}

/// Airspeeds in metres per second between which the rush of air grows from nothing to full.
const RUSH_SPEED: (f32, f32) = (6.0, 75.0);

/// Writes the flight gear into the scene: the rush of air by airspeed and by how loud the air
/// of the planet is, the boosters by their push and the flutter of the cloth. The gear
/// publishes nothing while nobody is on foot.
fn sense_flight(here: Res<Here>, flight: Option<Res<FlightFx>>, mut scene: ResMut<AudioScene>) {
    let fx = flight.as_deref().copied().unwrap_or_default();
    let air = here
        .0
        .as_ref()
        .map_or(1.0, |(_, spec)| loudness(&spec.record));
    scene.rush = smoothstep(RUSH_SPEED.0, RUSH_SPEED.1, fx.airspeed) * air;
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
            (
                sense_place,
                sense_listener,
                sense_shore,
                sense_flight,
                sense_companion,
            )
                .chain()
                .in_set(super::Sensing)
                .after(ShipSet::Place)
                .after(GearSet::Publish)
                .run_if(in_state(Screen::Gameplay)),
        );
}
