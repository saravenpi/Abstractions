//! Keeps the weather at the camera up to date a few times per second.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::rules::{Planet, Weather, forecast};
use crate::menus::HudWeather;
use crate::render::{GameCamera, LocalWeather};
use crate::space::{Anchor, Clock, Frame};
use crate::universe::{PlanetSpec, Universe};
use crate::world::{Biome, Planet as Streamed};

/// Seconds between two readings of the weather.
const INTERVAL: f32 = 0.25;

/// What the weather is read from: the frame, the universe, its time, the streamed planet and the camera.
#[derive(SystemParam)]
pub struct Sky<'w, 's> {
    /// Frame everything is rendered in.
    anchor: Option<Res<'w, Anchor>>,
    /// The universe of the world being played.
    universe: Option<Res<'w, Universe>>,
    /// Universe time.
    clock: Option<Res<'w, Clock>>,
    /// The planet the world streams, whose globe holds the climate of the moment.
    planet: Option<Res<'w, Streamed>>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// The planet under the camera and whether it is the home planet, kept so that neither is
/// worked out again at every reading.
#[derive(Default)]
pub struct Here(Option<(Universe, PlanetSpec, bool)>);

/// Returns the unit direction and the height above the sea level radius of render position `at` on a
/// planet of `radius` metres. The legacy flat world is a place on the equator.
fn place(anchor: &Anchor, radius: f64, at: Vec3) -> (DVec3, f32) {
    let pos = anchor.to_frame(at);
    match anchor.flat {
        true => (DVec3::X, at.y),
        false => (pos.normalize_or(DVec3::X), (pos.length() - radius) as f32),
    }
}

/// Returns the weather at the camera, or nothing away from any planet. The ground gives dust
/// only where it is bare, and the home planet keeps its calm opening.
fn read(sky: &Sky, here: &mut Here) -> Option<Weather> {
    let (anchor, universe) = (sky.anchor.as_deref()?, sky.universe.as_deref()?);
    let Frame::Surface(body) = anchor.frame else {
        return None;
    };
    let planet = sky.planet.as_deref().filter(|planet| planet.body == body)?;
    let camera = sky.camera.single().ok()?;
    let known = |(of, spec, _): &(Universe, PlanetSpec, bool)| of == universe && spec.id == body;
    if !here.0.as_ref().is_some_and(known) {
        here.0 = Some((*universe, universe.planet(body), body == universe.home()));
    }
    let (_, spec, home) = here.0.as_ref()?;
    let seconds = sky.clock.as_deref().map_or(0.0, |clock| clock.seconds);
    let (dir, altitude) = place(anchor, spec.radius, camera.translation());
    let air = planet.globe.air(dir, altitude);
    let base = Planet::of(&spec.record, spec.seed);
    let ground = Planet {
        bare: base.bare || planet.globe.biome(dir) == Biome::Desert,
        home: *home,
        ..base
    };
    Some(forecast(&ground, &air, seconds))
}

/// Reads the weather at the camera every [`INTERVAL`] seconds. Away from any planet the sky is
/// clear.
pub fn watch(
    time: Res<Time>,
    sky: Sky,
    mut here: Local<Here>,
    mut wait: Local<f32>,
    mut weather: ResMut<Weather>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 {
        return;
    }
    *wait = INTERVAL;
    weather.set_if_neq(read(&sky, &mut here).unwrap_or_default());
}

/// Clears the weather when the world is left, with what the sky and the HUD were told of it:
/// the title screen and the next world start under a sky nothing has reported on.
pub fn clear(
    mut weather: ResMut<Weather>,
    mut sky: ResMut<LocalWeather>,
    mut hud: ResMut<HudWeather>,
) {
    weather.set_if_neq(Weather::default());
    sky.set_if_neq(LocalWeather::default());
    hud.set_if_neq(HudWeather::default());
}
