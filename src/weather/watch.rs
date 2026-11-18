//! Keeps the weather at the camera up to date a few times per second.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::rules::{Planet, Weather, forecast};
use crate::climate::season;
use crate::climate::{self, Setting, Spot};
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Frame};
use crate::universe::{PlanetSpec, Universe};

/// Seconds between two readings of the weather.
const INTERVAL: f32 = 0.25;

/// What the weather is read from: the frame, the universe, its time and the camera.
#[derive(SystemParam)]
pub struct Sky<'w, 's> {
    /// Frame everything is rendered in.
    anchor: Option<Res<'w, Anchor>>,
    /// The universe of the world being played.
    universe: Option<Res<'w, Universe>>,
    /// Universe time.
    clock: Option<Res<'w, Clock>>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// The planet under the camera, kept so that its spec is not rebuilt at every reading.
#[derive(Default)]
pub struct Here(Option<(Universe, PlanetSpec)>);

/// Returns the place at render position `at` on the planet of `spec`, as the climate reads it.
/// The legacy flat world is a spot on the equator.
pub fn spot(anchor: &Anchor, spec: &PlanetSpec, at: Vec3) -> Spot {
    let pos = anchor.to_frame(at);
    let (dir, altitude) = match anchor.flat {
        true => (DVec3::X, at.y),
        false => (
            pos.normalize_or(DVec3::X),
            (pos.length() - spec.radius) as f32,
        ),
    };
    Spot {
        dir,
        altitude,
        inland: 0.0,
        slope: Vec2::ZERO,
    }
}

/// Returns the weather at `spot` on the planet of `spec` at `seconds` of universe time.
pub fn weather_at(spec: &PlanetSpec, spot: &Spot, seconds: f64) -> Weather {
    let climate = climate::at(&Setting::of(spec), spot, season(spec, seconds));
    forecast(&Planet::of(&spec.record, spec.seed), &climate, seconds)
}

/// Returns the weather at the camera, or nothing away from any planet.
fn read(sky: &Sky, here: &mut Here) -> Option<Weather> {
    let (anchor, universe) = (sky.anchor.as_deref()?, sky.universe.as_deref()?);
    let Frame::Surface(body) = anchor.frame else {
        return None;
    };
    let camera = sky.camera.single().ok()?;
    let known = |(of, spec): &(Universe, PlanetSpec)| of == universe && spec.id == body;
    if !here.0.as_ref().is_some_and(known) {
        here.0 = Some((*universe, universe.planet(body)));
    }
    let (_, spec) = here.0.as_ref()?;
    let seconds = sky.clock.as_deref().map_or(0.0, |clock| clock.seconds);
    let at = spot(anchor, spec, camera.translation());
    Some(weather_at(spec, &at, seconds))
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

/// Clears the weather when the world is left.
pub fn clear(mut weather: ResMut<Weather>) {
    weather.set_if_neq(Weather::default());
}
