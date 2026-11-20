//! The hearing: a few times a second, the calls of the herds and hunts in earshot of the listener,
//! and the alarms of the animals that flee a robot, sent to the audio as positioned cues.
//!
//! Contact calls and the cries of hunts come from `crate::life::signs::calls`, pure functions of the
//! herds and universe time, so every player hears them at the same moment. Alarms at a robot are
//! local, as the flight they come with is. Only the [`CALLS`] loudest calls of one hearing are sent.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{CALLS, EARSHOT, HALF_LEVEL, HEAR, QUIET};
use crate::audio::{AudioCue, Call};
use crate::fauna::beast::Beast;
use crate::fauna::herds::{Herds, herd_arc};
use crate::life::Life;
use crate::life::signs::calls::{Calling, Cry, contact, cries, voice};
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
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    beasts: Query<'w, 's, (&'static Beast, &'static Placed)>,
    cues: MessageWriter<'w, AudioCue>,
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
    loud: Vec<(f32, Call)>,
}

/// The listener: the camera and its place in the frame of the anchor.
pub struct Ear {
    /// Unit direction of the listener from the planet centre.
    pub dir: DVec3,
    /// Its place in the frame of the anchor.
    pub pos: DVec3,
    /// Its render transform.
    pub view: GlobalTransform,
}

/// Sends the calls heard since the last hearing.
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
        view: *camera,
    };
    gather(&h, &mut heard, (from, now));
    let Heard { calls, loud, .. } = &mut *heard;
    weigh(life, calls, (&ear, &h.anchor, globe.radius()), loud);
    for (_, call) in loud.iter() {
        h.cues.write(AudioCue::Call(*call));
    }
}

/// Fills the calls of `heard` with the contact calls, hunt cries and robot alarms within `span`.
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
    for herd in herds {
        if let Some(c) = contact(herd, span) {
            calls.push((c, herd.centre));
        }
    }
    for hunt in &h.herds.hunts {
        cries(hunt, span, |c| {
            calls.push((c, centre(c.herd).unwrap_or(hunt.to)))
        });
    }
    alarms(h, (calls, fled), span.1);
}

/// Adds to `calls` an alarm for each herd with an animal fleeing a robot at universe time `now`,
/// one per herd every [`ALARM_GAP`] seconds, as kept in `fled`.
fn alarms(h: &Hear, (calls, fled): Lists<'_>, now: f64) {
    fled.retain(|(_, at)| now - at < ALARM_GAP);
    for (beast, placed) in &h.beasts {
        let (herd, _) = beast.lead.key();
        if !beast.mood.fleeing || fled.iter().any(|(id, _)| *id == herd) {
            continue;
        }
        fled.push((herd, now));
        let species = beast.index;
        let calling = Calling {
            herd,
            species,
            cry: Cry::Alarm,
            at: now,
        };
        calls.push((calling, placed.pos.normalize()));
    }
}

/// Fills `loud` with the [`CALLS`] loudest of `calls` of `life` as heard by `ear`, on a planet of
/// `radius` metres seen through `anchor`, leaving out those out of earshot or too quiet.
pub fn weigh(
    life: &Life,
    calls: &[(Calling, DVec3)],
    (ear, anchor, radius): (&Ear, &Anchor, f64),
    loud: &mut Vec<(f32, Call)>,
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
        let toward = (from - ear.view.translation()).normalize_or_zero();
        let side = toward.dot(ear.view.right().as_vec3());
        let voice = voice(species, life.rung);
        loud.push((
            near,
            Call {
                voice,
                cry: calling.cry,
                side,
                near,
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
