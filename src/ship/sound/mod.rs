//! What every ship sounds like, for the audio: the thrust of its engines and its lift nozzles,
//! its warp drive, its gear and its canopy, the air it flies through, and where it is for the
//! listener. All of it is read from the real state of the hull, the one this machine flies and
//! the puppets of the other players alike, and written into the audio scene every frame.

mod near;
#[cfg(test)]
mod tests;

use bevy::ecs::message::Messages;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ShipSet;
use super::body::Ship;
use super::cockpit::{CANOPY_OPEN, Canopy, Cockpit, Lamp};
use super::exhaust::Exhaust;
use super::shake::Buffet;
use super::sky::Skies;
use crate::audio::{AudioCue, AudioScene, Ch, EARSHOT, HULLS, HullSound};
use crate::render::{GameCamera, Heat};
use crate::space::{Anchor, Frame, Location, Placed, Travel};
use crate::speed::{sound_speed, temperature_at};
use crate::universe::{PlanetSpec, heights};

/// Returns the speed of sound in metres per second `height` metres over the sea of `planet`,
/// from the gas of its air and its temperature at that height.
pub fn sound_at(planet: &PlanetSpec, height: f64) -> f32 {
    let record = &planet.record;
    let top = heights(record, planet.gravity, planet.radius)[2];
    let layer = (planet.scale_height(), top);
    let cold = temperature_at(record.temperature, record.mix.kappa(), layer, height);
    sound_speed(cold, &record.mix)
}

/// Returns what a ship sounds like from its exhaust and its cockpit alone, wherever it is: its
/// engines run while someone is aboard or while it flies.
pub fn engines(exhaust: &Exhaust, cockpit: &Cockpit) -> HullSound {
    let read = &cockpit.read;
    let running = cockpit.pilot.is_some() || !read.lamps[Lamp::Landed as usize];
    HullSound::default()
        .with(Ch::Present, 1.0)
        .with(Ch::Power, f32::from(u8::from(running)))
        .with(Ch::Thrust, exhaust.main)
        .with(Ch::Lift, exhaust.lift)
        .with(Ch::Warp, exhaust.warp)
        .with(Ch::Rcs, exhaust.rcs.abs().max_element())
        .with(Ch::Glow, exhaust.heat)
        .with(Ch::Air, exhaust.air)
        .with(Ch::Speed, read.speed.unwrap_or(0.0) as f32)
        .with(
            Ch::Height,
            read.height.map_or(1.0e4, |height| height as f32),
        )
        .with(Ch::Gear, cockpit.asked.gear)
}

/// What the listener and the air are read from.
#[derive(SystemParam)]
pub struct Ears<'w, 's> {
    /// Frame time.
    time: Res<'w, Time>,
    /// The frame of the render anchor.
    anchor: Res<'w, Anchor>,
    /// Whether the player is seated.
    location: Res<'w, Location>,
    /// The planet under the ship.
    skies: Res<'w, Skies>,
    /// The shake of the view.
    buffet: Res<'w, Buffet>,
    /// The heat of an entry, when the sky is drawn.
    heat: Option<Res<'w, Heat>>,
    /// The ears: the game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    /// The canopy of every ship.
    canopies: Query<'w, 's, (&'static ChildOf, &'static Canopy)>,
}

impl Ears<'_, '_> {
    /// Returns the speed of sound in metres per second in the air at `placed`: the one of the
    /// home planet away from any air, where it carries nothing anyway.
    fn sound(&self, placed: &Placed) -> f32 {
        match (self.anchor.frame, self.skies.planet()) {
            (Frame::Surface(_), Some(planet)) => {
                sound_at(planet, placed.pos.length() - planet.radius)
            }
            _ => HullSound::default().at(Ch::Sound),
        }
    }

    /// Returns `hull` placed for the listener: how far the ship at `placed` is, on which side,
    /// and where the listener stands around its exhaust. A ship out of earshot, or a world
    /// without a camera, gives a ship that is not there.
    fn place(&self, hull: HullSound, placed: &Placed) -> HullSound {
        let Ok(camera) = self.camera.single() else {
            return hull.with(Ch::Present, 0.0);
        };
        let at = self.anchor.to_render(placed.pos);
        let rear = self.anchor.rotation_to_render(placed.rot) * Vec3::Z;
        let to = camera.translation() - at;
        let toward = to.normalize_or_zero();
        hull.with(Ch::Present, f32::from(u8::from(to.length() <= EARSHOT)))
            .with(Ch::Distance, to.length())
            .with(Ch::Side, -toward.dot(camera.right().as_vec3()))
            .with(Ch::Aspect, rear.dot(toward))
            .with(Ch::Sound, self.sound(placed))
    }

    /// Returns how far the canopy of the ship `hull` is open, from 0 to 1.
    fn canopy(&self, hull: Entity) -> f32 {
        let hinge = self
            .canopies
            .iter()
            .find(|(parent, _)| parent.parent() == hull);
        hinge.map_or(0.0, |(_, canopy)| canopy.angle / CANOPY_OPEN.0)
    }

    /// Returns `hull` as the pilot hears it from the seat: from inside, ahead of the nozzles,
    /// with the shake of rough air and the heat of an entry on the airframe.
    fn board(&self, hull: HullSound) -> HullSound {
        let heat = self.heat.as_ref().map_or(0.0, |heat| heat.glow.heat);
        hull.with(Ch::Present, 1.0)
            .with(Ch::Inside, 1.0)
            .with(Ch::Distance, 0.0)
            .with(Ch::Side, 0.0)
            .with(Ch::Aspect, -1.0)
            .with(Ch::Rough, self.buffet.level.max(heat))
    }
}

/// Writes what every ship in earshot sounds like into the audio scene: first the ship this
/// machine flies, then the nearest of the others. A ship faster than sound booms as its shock
/// wave passes the listener.
pub fn feed(
    ears: Ears,
    ships: Query<(Entity, &Exhaust, &Cockpit, &Placed, Has<Ship>)>,
    mut scene: ResMut<AudioScene>,
    mut cues: ResMut<Messages<AudioCue>>,
    mut last: Local<[(f32, f32); HULLS]>,
) {
    let seated = ears.location.travel == Travel::InShip;
    let mut hulls = [HullSound::default(); HULLS];
    for (entity, exhaust, cockpit, placed, own) in &ships {
        let open = ears.canopy(entity);
        let hull = ears.place(engines(exhaust, cockpit), placed);
        let hull = hull.with(Ch::Canopy, open);
        match (own, seated) {
            (true, true) => hulls[0] = ears.board(hull),
            (true, false) => hulls[0] = hull,
            (false, _) => near::admit(&mut hulls[1..], hull),
        }
    }
    for strength in near::booms(&hulls, &mut last, ears.time.delta_secs()) {
        cues.write(AudioCue::Boom(strength));
    }
    if scene.hulls != hulls {
        scene.hulls = hulls;
    }
}

/// Registers the feed, which runs with the other reports of the ship where the audio exists.
pub(super) fn plugin(app: &mut App) {
    let heard = resource_exists::<AudioScene>.and_then(resource_exists::<Messages<AudioCue>>);
    app.add_systems(Update, feed.in_set(ShipSet::Report).run_if(heard))
        .add_systems(
            Update,
            near::decks
                .in_set(ShipSet::Report)
                .run_if(resource_exists::<crate::avatar::tread::Hulls>),
        );
}
