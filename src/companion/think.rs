//! The thinking system: watches the player, runs the mind and the face, and sends the chirps.

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
use super::tune::{
    BANK_SHARE, DELIGHT_SECONDS, FLIGHT_RATE, SLEEPY_NIGHT, TAKEOFF_SECONDS, WAKE_SECONDS,
};
use crate::computer::Atlas;
use crate::ship::ShipEvent;
use crate::universe::BodyId;

/// Seconds the companion waits before it says the same thing again, and before it says
/// anything else, so it never chatters.
const QUIET: (f32, f32) = (10.0, 1.5);

/// Head tilt in radians of the curious face.
const CURIOUS_TILT: f32 = 0.17;

/// Turns something that happened in the mind into a reaction of the face and a chirp.
fn react(
    cue: Cue,
    brain: &mut Brain,
    face: &mut Face,
    sense: &CompanionSense,
) -> Option<ChirpKind> {
    match cue {
        Cue::Woke => {
            face.react(Expression::Surprised, WAKE_SECONDS);
            Some(ChirpKind::Wake)
        }
        Cue::Started(Idle::Inspect) => {
            brain.choose_inspect(sense);
            None
        }
        Cue::Started(Idle::Hum) => Some(ChirpKind::Hum),
        Cue::Started(_) | Cue::Dozed => None,
    }
}

/// Returns the face the surroundings and the idle in progress call for.
fn wanted(brain: &Brain, sense: &CompanionSense) -> Expression {
    let idle = match (brain.mind.dozing, brain.mind.idle) {
        (true, _) => Some(Expression::Sleepy),
        (false, idle) => idle.map(Idle::face),
    };
    ambient(sense, idle, brain.nearby(sense).is_some())
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

    /// Reacts to what just happened around the companion: a delighted face for a discovery,
    /// a happy one with a delighted chirp when the player takes off with the gear, and a
    /// wide-eyed one with its chirp when the ship lifts off.
    fn stir(&mut self, brain: &mut Brain, face: &mut Face, body: BodyId) -> Option<ChirpKind> {
        let sense = *self.sense;
        let fresh = self.first_steps(body);
        if brain.discovered(&sense, fresh) {
            face.react(Expression::Love, DELIGHT_SECONDS);
        }
        let took_off = sense.flying && !brain.aloft;
        brain.aloft = sense.flying;
        if took_off {
            face.react(Expression::Happy, TAKEOFF_SECONDS);
            return Some(ChirpKind::Delight);
        }
        if !self.liftoff() {
            return None;
        }
        face.react(Expression::Surprised, TAKEOFF_SECONDS);
        Some(ChirpKind::Surprised)
    }
}

/// Moves the attention on by `dt` seconds, points the eyes and poses the body for the idle in
/// progress and the face shown, and banks it with a flying player.
fn settle(brain: &mut Brain, face: &Face, bank: f32, dt: f32) {
    let scene = brain.scene;
    brain.attention.advance(&scene, dt);
    brain.look = brain.focus();
    brain.pose = brain.mind.pose();
    if face.shown() == Expression::Curious {
        brain.pose.roll += CURIOUS_TILT;
    }
    brain.pose.roll -= bank * BANK_SHARE * brain.flight;
}

/// Watches the player, advances the idle mind, the attention and the face, and sends the
/// chirps, with a pause after each one.
pub fn think(
    (around, sights): (Around, Sights),
    mut events: Events,
    bot: Single<(&mut Brain, &mut Face, &Transform, &Float), With<Companion>>,
    mut said: Local<Option<(ChirpKind, f32)>>,
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
    let mut heard = cue.and_then(|cue| react(cue, &mut brain, &mut face, &sense));
    heard = heard.or(events.stir(&mut brain, &mut face, around.location.body));
    let changed = face.advance(wanted(&brain, &sense), dt);
    heard = heard.or(changed.and_then(chirp));
    settle(&mut brain, &face, sense.bank, dt);
    let last = said.map(|(kind, ago)| (kind, ago + dt));
    *said = last;
    let wait = |kind: ChirpKind| match last {
        Some((same, ago)) if same == kind => ago < QUIET.0,
        Some((_, ago)) => ago < QUIET.1,
        None => false,
    };
    if let Some(kind) = heard.filter(|kind| !wait(*kind)) {
        *said = Some((kind, 0.0));
        events.chirps.write(CompanionChirp(kind));
    }
}
