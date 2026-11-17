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
use super::sense::{Around, CompanionSense};
use super::tune::{DELIGHT_SECONDS, SLEEPY_NIGHT, STILL_SPEED, TAKEOFF_SECONDS, WAKE_SECONDS};
use crate::computer::Atlas;
use crate::ship::ShipEvent;
use crate::universe::BodyId;

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
    /// a wide-eyed one with its chirp when the ship lifts off.
    fn stir(&mut self, brain: &mut Brain, face: &mut Face, body: BodyId) -> Option<ChirpKind> {
        let sense = *self.sense;
        let fresh = self.first_steps(body);
        if brain.discovered(&sense, fresh) {
            face.react(Expression::Love, DELIGHT_SECONDS);
        }
        if !self.liftoff() {
            return None;
        }
        face.react(Expression::Surprised, TAKEOFF_SECONDS);
        Some(ChirpKind::Surprised)
    }
}

/// Points the eyes and poses the body for the idle in progress and the face shown.
fn settle(brain: &mut Brain, face: &Face) {
    brain.look = brain.focus();
    brain.pose = brain.mind.pose();
    if face.shown() == Expression::Curious {
        brain.pose.roll += CURIOUS_TILT;
    }
}

/// Watches the player, advances the idle mind, the attention and the face, and sends the chirps.
pub fn think(
    around: Around,
    mut events: Events,
    bot: Single<(&mut Brain, &mut Face), With<Companion>>,
) {
    let (mut brain, mut face) = bot.into_inner();
    let (dt, sense) = (around.dt(), *events.sense);
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
    heard = heard.or(events.stir(&mut brain, &mut face, around.location.body));
    let changed = face.advance(wanted(&brain, &sense), dt);
    heard = heard.or(changed.and_then(chirp));
    settle(&mut brain, &face);
    if let Some(kind) = heard {
        events.chirps.write(CompanionChirp(kind));
    }
}
