//! The thinking system: watches the player, runs the mind and the face, and sends the chirps the
//! speech lets through.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::face::Expression;
use super::face::machine::Face;
use super::idle::Idle;
use super::mind::{Cue, Input};
use super::mood::{ChirpKind, CompanionChirp, ambient, chirp};
use super::motion::{Float, blend};
use super::sense::{Around, CompanionSense};
use super::sights::Sights;
use super::speech::{Said, Speech, Why, reason, spoken};
use super::tune::{DELIGHT_SECONDS, FLIGHT_RATE, SLEEPY_NIGHT, TAKEOFF_SECONDS, WAKE_SECONDS};
use crate::computer::Atlas;
use crate::settings::{Chatter, Persistent, Settings};
use crate::ship::ShipEvent;
use crate::universe::BodyId;

/// Head tilt in radians of the curious face.
const CURIOUS_TILT: f32 = 0.17;

/// Turns something that happened in the mind into a reaction of the face and what it would say.
fn react(cue: Cue, brain: &mut Brain, face: &mut Face) -> Said {
    match cue {
        Cue::Woke => face.react(Expression::Surprised, WAKE_SECONDS),
        Cue::Started(Idle::Inspect) => brain.choose_inspect(),
        Cue::Started(_) | Cue::Dozed => {}
    }
    spoken(cue)
}

/// Returns the face the surroundings and the idle in progress call for.
fn wanted(brain: &Brain, sense: &CompanionSense) -> Expression {
    let idle = match (brain.mind.dozing, brain.mind.idle) {
        (true, _) => Some(Expression::Sleepy),
        (false, idle) => idle.map(Idle::face),
    };
    ambient(sense, idle)
}

/// What the thinking reads and writes besides the companion itself.
#[derive(SystemParam)]
pub struct Events<'w, 's> {
    /// What the companion senses.
    sense: Res<'w, CompanionSense>,
    /// What the player has discovered, with the planets already walked on.
    atlas: Option<ResMut<'w, Atlas>>,
    /// What happens to the ship.
    ship: MessageReader<'w, 's, ShipEvent>,
    /// Where the chirps go.
    chirps: MessageWriter<'w, CompanionChirp>,
    /// How often the player lets the companion speak.
    settings: Option<Res<'w, Persistent<Settings>>>,
}

impl Events<'_, '_> {
    /// Returns true when the ship just left the ground with the player in it.
    fn liftoff(&mut self) -> bool {
        let left = self.ship.read().any(|event| *event == ShipEvent::Liftoff);
        left && self.sense.seated
    }

    /// Returns true on the first steps on the planet `body`, and records them in the atlas.
    fn first_steps(&mut self, body: BodyId) -> bool {
        let Some(atlas) = self.atlas.as_mut() else {
            return false;
        };
        let landed = !self.sense.seated && !atlas.walked.contains(&body);
        landed && atlas.walk(body)
    }

    /// Reacts to what just happened around the companion: a delighted face and chirp for a
    /// discovery, a happy face with a delighted chirp when the player takes off with the gear, and
    /// a wide-eyed one with its chirp when the ship lifts off.
    fn stir(&mut self, brain: &mut Brain, face: &mut Face, body: BodyId) -> Said {
        let sense = *self.sense;
        let fresh = self.first_steps(body);
        let found = brain.discovered(&sense, fresh);
        if found {
            face.react(Expression::Love, DELIGHT_SECONDS);
        }
        let took_off = sense.flying && !brain.aloft;
        brain.aloft = sense.flying;
        let lifted = self.liftoff();
        let (kind, face_of) = match (took_off, lifted, found) {
            (true, ..) => (ChirpKind::Delight, Expression::Happy),
            (_, true, _) => (ChirpKind::Surprised, Expression::Surprised),
            (_, _, true) => return Some((ChirpKind::Delight, Why::Event)),
            _ => return None,
        };
        face.react(face_of, TAKEOFF_SECONDS);
        Some((kind, Why::Event))
    }

    /// Returns how often the companion may speak, and whether the player is in the air or the ship.
    fn voice(&self) -> (Chatter, bool) {
        let voice = self
            .settings
            .as_ref()
            .map_or(Chatter::Rare, |s| s.audio.companion);
        (voice, self.sense.flying || self.sense.seated)
    }
}

/// Moves the attention on by `dt` seconds, points the eyes and poses the body for the idle in
/// progress and the face shown. Its bank in flight is its own: see `soar`.
fn settle(brain: &mut Brain, face: &Face, dt: f32) {
    let scene = brain.scene;
    brain.attention.advance(&scene, dt);
    brain.look = brain.focus();
    brain.pose = brain.mind.pose();
    if face.shown() == Expression::Curious {
        brain.pose.roll += CURIOUS_TILT;
    }
}

/// Watches the player, advances the idle mind, the attention and the face, and sends the chirps
/// the speech lets through.
pub fn think(
    (around, sights): (Around, Sights),
    mut events: Events,
    bot: Single<(&mut Brain, &mut Face, &Transform, &Float), With<Companion>>,
    mut speech: Local<Speech>,
) {
    let (mut brain, mut face, at, float) = bot.into_inner();
    let (dt, sense) = (around.dt(), *events.sense);
    brain.observe(&around.anchor, around.stance(), dt);
    brain.scene = sights.scene(&around, at, float, brain.drift);
    brain.side = around.body.right().as_vec3();
    let flying = f32::from(u8::from(sense.flying));
    brain.flight += (flying - brain.flight) * blend(FLIGHT_RATE, dt);
    let input = Input {
        speed: brain.drift.length(),
        night: sense.night > SLEEPY_NIGHT,
        seated: sense.seated,
        dt,
    };
    let cue = brain.mind.advance(&input);
    let mut heard = cue.and_then(|cue| react(cue, &mut brain, &mut face));
    heard = heard.or(events.stir(&mut brain, &mut face, around.location.body));
    let changed = face.advance(wanted(&brain, &sense), dt);
    heard = heard.or(changed
        .and_then(chirp)
        .map(|kind| (kind, reason(kind, &sense))));
    settle(&mut brain, &face, dt);
    if let Some(kind) = speech.hear(heard, events.voice(), dt) {
        events.chirps.write(CompanionChirp(kind));
    }
}
