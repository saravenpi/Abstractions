//! What every ship sounds like, for the audio: the thrust of its engines and its lift nozzles,
//! its warp drive, its gear and its canopy, the air it flies through, and where it is for the
//! listener. All of it is read from the real state of the hull, the one this machine flies and
//! the puppets of the other players alike, and written into the audio scene every frame.

mod flow;
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
use crate::audio::{AudioScene, Ch, EARSHOT, HULLS, HullSound, Listener, Sound};
use crate::render::{GameCamera, Heat};
use crate::space::{Anchor, Frame, Location, Placed, Travel};
use crate::speed::sound_at;
use crate::world::Terrain;

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
    /// The ears of the player, where the audio has them.
    listener: Option<Res<'w, Listener>>,
    /// The game camera: the ears where the audio has none.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    /// The canopy of every ship.
    canopies: Query<'w, 's, (&'static ChildOf, &'static Canopy)>,
    /// The ground and the sea under the ships.
    terrain: Option<Res<'w, Terrain>>,
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

    /// Returns where the ears are and which way they face: the listener of the audio, or the
    /// camera without one.
    fn ears(&self) -> Option<(Vec3, Quat)> {
        match self.listener.as_deref() {
            Some(ear) => Some((ear.at, ear.turn)),
            None => self
                .camera
                .single()
                .ok()
                .map(|c| (c.translation(), c.rotation())),
        }
    }

    /// Returns `hull` placed for the listener: how far the ship at `placed` is, its bearing, and
    /// where the listener stands around its exhaust. A ship out of earshot, or a world without
    /// ears, gives a ship that is not there.
    fn place(&self, hull: HullSound, placed: &Placed) -> HullSound {
        let Some((ear, turn)) = self.ears() else {
            return hull.with(Ch::Present, 0.0);
        };
        let at = self.anchor.to_render(placed.pos);
        let rear = self.anchor.rotation_to_render(placed.rot) * Vec3::Z;
        let surface = matches!(self.anchor.frame, Frame::Surface(_));
        let wet = self
            .terrain
            .as_deref()
            .filter(|_| surface)
            .is_some_and(|terrain| {
                let chart = terrain.locate(placed.pos);
                terrain.height(chart.x, chart.z) < terrain.globe().sea_level()
            });
        let to = ear - at;
        let toward = to.normalize_or_zero();
        hull.with(Ch::Present, f32::from(u8::from(to.length() <= EARSHOT)))
            .with(Ch::Distance, to.length())
            .with(Ch::Side, -toward.dot(turn * Vec3::X))
            .with(Ch::Front, toward.dot(turn * Vec3::Z))
            .with(Ch::Aspect, rear.dot(toward))
            .with(Ch::Sound, self.sound(placed))
            .with(Ch::Wet, f32::from(u8::from(wet)))
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
            .with(Ch::Front, 0.0)
            .with(Ch::Aspect, -1.0)
            .with(Ch::Rough, self.buffet.level.max(heat))
            .with(Ch::Roar, heat)
    }
}

/// Writes what every ship in earshot sounds like into the audio scene: first the ship this
/// machine flies, then the nearest of the others. The ship the player sits in is heard from the
/// seat while the ears are in its cabin, and from the chase view riding along otherwise. A ship
/// faster than sound booms from where it is as its shock wave passes the listener.
pub fn feed(
    ears: Ears,
    ships: Query<(Entity, &Exhaust, &Cockpit, &Placed, Option<&Ship>)>,
    mut scene: ResMut<AudioScene>,
    mut sounds: ResMut<Messages<Sound>>,
    mut last: Local<[(f32, f32); HULLS]>,
) {
    let seated = ears.location.travel == Travel::InShip;
    let cabin = seated && ears.listener.as_deref().is_none_or(|ear| ear.cabin);
    let (mut hulls, mut seat) = ([HullSound::default(); HULLS], None);
    for (entity, exhaust, cockpit, placed, body) in &ships {
        let hull = ears.place(engines(exhaust, cockpit), placed);
        let hull = hull.with(Ch::Canopy, ears.canopy(entity));
        let (own, hull) = match body {
            Some(ship) => (true, flow::flow(hull, ship)),
            None => (false, hull),
        };
        seat = if own {
            Some(ears.anchor.to_render(placed.pos))
        } else {
            seat
        };
        match (own, cabin) {
            (true, true) => hulls[0] = ears.board(hull),
            (true, false) => hulls[0] = hull.with(Ch::Riding, f32::from(u8::from(seated))),
            (false, _) => near::admit(&mut hulls[1..], hull),
        }
    }
    let dt = ears.time.delta_secs();
    near::sonic((ears.ears(), &hulls), &mut last, dt, &mut sounds);
    if (scene.hulls, scene.cockpit) != (hulls, seat) {
        (scene.hulls, scene.cockpit) = (hulls, seat);
    }
}

/// Registers the feed, which runs with the other reports of the ship where the audio exists.
pub(super) fn plugin(app: &mut App) {
    let heard = resource_exists::<AudioScene>.and_then(resource_exists::<Messages<Sound>>);
    app.add_systems(Update, feed.in_set(ShipSet::Report).run_if(heard))
        .add_systems(
            Update,
            near::decks
                .in_set(ShipSet::Report)
                .run_if(resource_exists::<crate::avatar::tread::Hulls>),
        );
}
