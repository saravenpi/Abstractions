//! Keeps the weather at the camera up to date, every frame: the cloud field is smooth in place
//! and in time, so what is read from it never steps.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::field::{Field, Overhead};
use super::grid::Patch;
use super::rules::{Planet, Weather, forecast, under};
use super::wet::Wet;
use crate::menus::HudWeather;
use crate::render::{GameCamera, LocalWeather};
use crate::space::{Anchor, Clock, Frame};
use crate::universe::{BodyId, PlanetSpec, Universe};
use crate::world::{Biome, Globe, Planet as Streamed};

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

/// The planet under the camera, kept so that nothing about it is worked out again every frame.
pub struct Known {
    /// The universe it belongs to.
    universe: Universe,
    /// The planet or moon.
    spec: PlanetSpec,
    /// Its cloud field.
    field: Field,
    /// What the weather rules read of it, over the ground its class has everywhere.
    ground: Planet,
    /// The wetness of the ground at the texels of the cover grid around the camera.
    wet: Wet,
}

/// The planet the weather was last read on.
#[derive(Default)]
pub struct Here(Option<Known>);

impl Here {
    /// Returns what is kept about planet `body` of `universe`, worked out when it changes.
    fn on(&mut self, universe: &Universe, body: BodyId) -> &mut Known {
        let same = |known: &Known| known.universe == *universe && known.spec.id == body;
        if !self.0.as_ref().is_some_and(same) {
            self.0 = None;
        }
        self.0.get_or_insert_with(|| {
            let spec = universe.planet(body);
            Known {
                universe: *universe,
                field: Field::of(&spec, body == universe.home()),
                ground: Planet::of(&spec.record),
                wet: Wet::default(),
                spec,
            }
        })
    }
}

impl Known {
    /// Returns what the cloud field says over the unit direction `dir` of `globe`, `altitude`
    /// metres above sea level, at `seconds` of universe time.
    fn overhead(&mut self, globe: &Globe, place: (DVec3, f32), seconds: f64) -> Overhead {
        let (dir, altitude) = place;
        let patch = Patch::around(dir);
        let wet = self.wet.over(globe, &patch);
        let (base, top) = (self.spec.cloud_base as f32, self.spec.cloud_top as f32);
        Overhead {
            cover: self.field.cover(&patch, wet, seconds),
            storm: self.field.storm(dir, seconds),
            share: self.field.share(seconds),
            under: under(altitude, base, top),
        }
    }
}

/// Returns the unit direction and the height above the sea level radius of render position `at` on a
/// planet of `radius` metres. The legacy flat world is a place on the equator.
fn place(anchor: &Anchor, radius: f64, at: Vec3) -> (DVec3, f32) {
    let pos = anchor.to_frame(at);
    match anchor.flat {
        true => (DVec3::X, at.y),
        false => (pos.normalize_or(DVec3::X), (pos.length() - radius) as f32),
    }
}

/// Returns the weather at the camera, or nothing away from any planet: the cloud field over
/// the place, the climate of the air there, and dust only where the ground is bare.
fn read(sky: &Sky, here: &mut Here) -> Option<Weather> {
    let (anchor, universe) = (sky.anchor.as_deref()?, sky.universe.as_deref()?);
    let Frame::Surface(body) = anchor.frame else {
        return None;
    };
    let planet = sky.planet.as_deref().filter(|planet| planet.body == body)?;
    let camera = sky.camera.single().ok()?;
    let known = here.on(universe, body);
    let seconds = sky.clock.as_deref().map_or(0.0, |clock| clock.seconds);
    let (dir, altitude) = place(anchor, known.spec.radius, camera.translation());
    let air = planet.globe.air(dir, altitude);
    let overhead = known.overhead(&planet.globe, (dir, altitude), seconds);
    let ground = Planet {
        bare: known.ground.bare || planet.globe.biome(dir) == Biome::Desert,
        ..known.ground
    };
    Some(forecast(&ground, &air, &overhead))
}

/// Reads the weather at the camera every frame. Away from any planet the sky is clear.
pub fn watch(sky: Sky, mut here: Local<Here>, mut weather: ResMut<Weather>) {
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
