//! The hearing: a few times a second, the calls of the herds and hunts in earshot of the listener,
//! and the alarms of the animals that flee a robot, sent to the audio as sounds from where they call.
//!
//! Contact calls and the cries of hunts come from `crate::life::signs::calls`, pure functions of the
//! herds and universe time, so every player hears them at the same moment. Alarms at a robot are
//! local, as the flight they come with is; so are the threats of bold animals that stand their
//! ground. Herds call more around sunrise and sunset, by the daylight at their home. Every call heard
//! is kept for the singing, which moves the bodies that make it; only the [`CALLS`] loudest calls of
//! one hearing are sent to the audio.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::sing::Sung;
use super::tune::{CALLS, EARSHOT, HALF_LEVEL, HEAR, QUIET};
use crate::audio::spatial::tune::CALL;
use crate::audio::{AudioCue, Call, Origin, Sound};
use crate::fauna::beast::Beast;
use crate::fauna::gate::Spec;
use crate::fauna::herds::{Herds, herd_arc};
use crate::fauna::senses::Stance;
use crate::fauna::site;
use crate::life::Life;
use crate::life::signs::calls::{Calling, Cry, chorus, contact, cries, distress, rhythm, voice};
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Placed};
use crate::world::Terrain;

/// Seconds between two alarms at a robot of one herd.
const ALARM_GAP: f64 = 3.0;

/// The calls of a hearing and the herds that called an alarm at a robot, and when.
type Lists<'a> = (&'a mut Vec<(Calling, DVec3)>, &'a mut Vec<(u64, f64)>);

/// What the hearing reads and writes.
#[derive(SystemParam)]
pub struct Hear<'w, 's> {
    clock: Res<'w, Clock>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    herds: Res<'w, Herds>,
    spec: Res<'w, Spec>,
    sung: ResMut<'w, Sung>,
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    beasts: Query<'w, 's, (&'static Beast, &'static Placed)>,
    sounds: MessageWriter<'w, Sound>,
}

/// What the hearing keeps from one hearing to the next, so it never allocates once warm.
#[derive(Default)]
pub struct Heard {
    /// Universe time of the last hearing.
    last: f64,
    /// Calls of this hearing, as callings and where they come from.
    calls: Vec<(Calling, DVec3)>,
    /// The herds that called an alarm at a robot, and when.
    fled: Vec<(u64, f64)>,
    /// Calls to send, with the level they arrive at.
    loud: Vec<(f32, Sound)>,
}

/// The listener: the place of the camera in the frame of the anchor. The audio places each call
/// for the ears itself; the hearing only picks which calls are worth sending.
pub struct Ear {
    /// Unit direction of the listener from the planet centre.
    pub dir: DVec3,
    /// Its place in the frame of the anchor.
    pub pos: DVec3,
}

/// Keeps the calls heard since the last hearing for the singing and sends the loudest.
pub fn hear(mut h: Hear, mut heard: Local<Heard>) {
    let globe = h.terrain.globe().clone();
    let (Some(life), Ok(camera)) = (globe.life(), h.camera.single()) else {
        return;
    };
    let now = h.clock.seconds;
    let from = if now - heard.last > 4.0 * f64::from(HEAR) || now < heard.last {
        now - f64::from(HEAR)
    } else {
        heard.last
    };
    heard.last = now;
    let pos = h.anchor.to_frame(camera.translation());
    let ear = Ear {
        dir: pos.normalize(),
        pos,
    };
    gather(&h, &mut heard, (from, now));
    let Heard { calls, loud, .. } = &mut *heard;
    let tempo = |c: &Calling| {
        let species = life.species.get(usize::from(c.species))?;
        Some(rhythm(&voice(species, life.rung), c.cry))
    };
    h.sung.remember(now, calls.iter().map(|(c, _)| *c), tempo);
    weigh(life, calls, (&ear, &h.anchor, globe.radius()), loud);
    for (_, sound) in loud.iter() {
        h.sounds.write(*sound);
    }
}

/// Fills the calls of `heard` with the contact calls, hunt cries and robot alarms and threats and
/// the distress of the hurt, within `span`.
fn gather(h: &Hear, heard: &mut Heard, span: (f64, f64)) {
    let Heard { calls, fled, .. } = heard;
    calls.clear();
    let herds = &h.herds.herds;
    let centre = |id: u64| {
        herds
            .iter()
            .find(|herd| herd.id == id)
            .map(|herd| herd.centre)
    };
    let planet = h.spec.planet.as_deref();
    for herd in herds {
        let dawn = |t: f64| planet.map_or(0.0, |p| chorus(site::light(p, herd.home, t)));
        if let Some(c) = contact(herd, &dawn, span) {
            calls.push((c, herd.centre));
        }
    }
    for hunt in &h.herds.hunts {
        cries(hunt, span, |c| {
            calls.push((c, centre(c.herd).unwrap_or(hunt.to)))
        });
    }
    alarms(h, (calls, fled), span.1);
    cries_of_pain(h, calls, span);
}

/// Adds to `calls` the distress cries of the hurt animals drawn, within the universe times `span`.
fn cries_of_pain(h: &Hear, calls: &mut Vec<(Calling, DVec3)>, span: (f64, f64)) {
    for (beast, placed) in &h.beasts {
        let hurt = 1.0 - beast.wits.health.share();
        let (herd, slot) = beast.lead.key();
        let who = (herd, beast.index, slot);
        if let Some(c) = distress(who, hurt, span) {
            calls.push((c, placed.pos.normalize()));
        }
    }
}

/// Adds to `calls` an alarm for each herd with an animal fleeing, or a threat for one standing its
/// ground, at universe time `now`, one per herd every [`ALARM_GAP`] seconds, as kept in `fled`.
fn alarms(h: &Hear, (calls, fled): Lists<'_>, now: f64) {
    fled.retain(|(_, at)| now - at < ALARM_GAP);
    for (beast, placed) in &h.beasts {
        let (herd, slot) = beast.lead.key();
        let cry = match (beast.mood.fleeing, beast.wits.stance) {
            (true, _) => Cry::Alarm,
            (false, Stance::Threaten) => Cry::Threat,
            _ => continue,
        };
        if fled.iter().any(|(id, _)| *id == herd) {
            continue;
        }
        fled.push((herd, now));
        let calling = Calling {
            herd,
            species: beast.index,
            slot,
            cry,
            at: now,
        };
        calls.push((calling, placed.pos.normalize()));
    }
}

/// Fills `loud` with the [`CALLS`] loudest of `calls` of `life` as heard by `ear`, on a planet of
/// `radius` metres seen through `anchor`, leaving out those out of earshot or too quiet. Each is a
/// sound from where the animals call, at the height of the ears, whose reach grows with the size
/// of the animal.
pub fn weigh(
    life: &Life,
    calls: &[(Calling, DVec3)],
    (ear, anchor, radius): (&Ear, &Anchor, f64),
    loud: &mut Vec<(f32, Sound)>,
) {
    loud.clear();
    for (calling, dir) in calls {
        let Some(species) = life.species.get(usize::from(calling.species)) else {
            continue;
        };
        let metres = herd_arc(*dir, ear.dir) * radius;
        let near = level(species.history.mass, metres as f32);
        if metres > EARSHOT || near < QUIET || species.body.parts.is_empty() {
            continue;
        }
        let from = anchor.to_render(*dir * ear.pos.length());
        let call = Call {
            voice: voice(species, life.rung),
            cry: calling.cry,
        };
        let reach = CALL * HALF_LEVEL * species.history.mass.max(1e-3).cbrt();
        loud.push((
            near,
            Sound {
                cue: AudioCue::Call(call),
                from: Origin::At(from, reach),
            },
        ));
    }
    loud.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
    loud.truncate(CALLS);
}

/// Returns the level, 0 to 1, a call of an animal of `mass` kilograms arrives at from `metres` away:
/// half at [`HALF_LEVEL`] times the cube root of the mass, falling as one over the distance past it.
pub fn level(mass: f32, metres: f32) -> f32 {
    let half = HALF_LEVEL * mass.max(1e-3).cbrt();
    half / (half + metres.max(0.0))
}
