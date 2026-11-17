//! The thinking system: watches the player, runs the mind and the face, and sends the chirps.

use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::face::Expression;
use super::face::machine::Face;
use super::idle::Idle;
use super::mind::{Cue, Input};
use super::mood::{ChirpKind, CompanionChirp, ambient, chirp};
use super::sense::{Around, CompanionSense};
use super::tune::{DELIGHT_SECONDS, SLEEPY_NIGHT, STILL_SPEED, WAKE_SECONDS};

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

/// Watches the player, advances the idle mind, the attention and the face, and sends the chirps.
pub fn think(
    around: Around,
    sense: Res<CompanionSense>,
    bot: Single<(&mut Brain, &mut Face), With<Companion>>,
    mut chirps: MessageWriter<CompanionChirp>,
) {
    let (mut brain, mut face) = bot.into_inner();
    let dt = around.dt();
    let turn = brain.observe(&around.anchor, around.stance(), dt);
    let speed = brain.drift.length();
    let input = Input {
        speed,
        night: sense.night > SLEEPY_NIGHT,
        seated: sense.seated,
        dt,
    };
    let cue = brain.mind.advance(&input);
    brain.attention.advance(turn, speed < STILL_SPEED, dt);
    let mut heard = cue.and_then(|cue| react(cue, &mut brain, &mut face, &sense));
    if brain.discovered(&sense, around.location.body) {
        face.react(Expression::Love, DELIGHT_SECONDS);
    }
    let changed = face.advance(wanted(&brain, &sense), dt);
    heard = heard.or(changed.and_then(chirp));
    brain.look = brain.focus();
    brain.pose = brain.mind.pose();
    if face.shown() == Expression::Curious {
        brain.pose.roll += CURIOUS_TILT;
    }
    if let Some(kind) = heard {
        chirps.write(CompanionChirp(kind));
    }
}
