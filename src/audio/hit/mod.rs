//! Blows heard: the game sends an [`Impact`] where a blow lands and a [`Whoosh`] where a tool is
//! swung, and this module turns each into a [`Sound`] from that place.
//!
//! The sound follows the contact: loud with the logarithm of the energy, ringing at a pitch that
//! follows the stiffness and the mass of what was struck and of the tool, a short wet noise only when
//! the blow cut, a high tick when it glanced off. A swing whooshes louder and lower the heavier the
//! tool is. At most three blips sound for a blow, and blows leave [`super::cues`] voices free for
//! the rest.

mod impact;
mod pack;
mod sound;
pub mod tune;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use impact::{Body, Impact, Outer, Whoosh};
pub use sound::{HitSound, Kind, of_impact, of_whoosh};

use super::cue::AudioCue;
use super::spatial::tune::{HIT, TOUCH};
use super::spatial::{Listener, Origin, Sound};

/// Returns where a blow at `at` starts for `listener`: held when it lands within [`TOUCH`] of the
/// listener's body, which then carries it to the ears as much as the air around it is thin.
pub fn origin(listener: &Listener, at: Vec3) -> Origin {
    let touches = listener.body.is_some_and(|body| body.distance(at) <= TOUCH);
    match touches {
        true => Origin::Held(at, HIT.0, HIT.1 * (1.0 - listener.air.density.min(1.0))),
        false => Origin::At(at, HIT.0),
    }
}

/// Plays every blow that landed, from where it landed.
fn strike(
    mut impacts: MessageReader<Impact>,
    listener: Res<Listener>,
    mut sounds: MessageWriter<Sound>,
) {
    for impact in impacts.read() {
        sounds.write(Sound {
            cue: AudioCue::Hit(of_impact(impact)),
            from: origin(&listener, impact.at),
        });
    }
}

/// Plays every tool swung, from its tip. A swing is the noise of the air it moves, so the body
/// never carries it.
fn swing(mut swings: MessageReader<Whoosh>, mut sounds: MessageWriter<Sound>) {
    for swing in swings.read() {
        sounds.write(Sound {
            cue: AudioCue::Hit(of_whoosh(swing)),
            from: Origin::At(swing.at, HIT.0),
        });
    }
}

/// Registers the messages of blows and the systems that play them.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<Impact>()
        .add_message::<Whoosh>()
        .add_systems(
            Update,
            (
                strike.run_if(on_message::<Impact>),
                swing.run_if(on_message::<Whoosh>),
            ),
        );
}
