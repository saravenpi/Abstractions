//! The body in the wind-up: before an attack the whole body prepares, not only the part that makes
//! it, and after it the body is off balance.
//!
//! The hips drop by a share of the hip height that grows with the commitment of the attack, the weight
//! moves back, a trample or a downward swipe rears, a charge or a lunge ducks its head end, a tail
//! sweep turns the flank. The blow throws the weight forward and the recovery shows it off balance
//! for as long as the animal is open, then everything is back at ease.

use super::kinds::Kind;
use super::timing::Attacking;
use super::tune::{COMMIT, CROUCH, DUCK, FLANK, OFF_BALANCE, REAR, RECOVER, WEIGHT};
use crate::fauna::beast::Beast;
use crate::fauna::motion::brace::Brace;

/// Returns a smooth rise from 0 to 1 over `t` from 0 to 1.
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns how loaded the body is and how far its weight is thrown, each 0 to 1, `t` seconds into an
/// attack of `wind` seconds of wind-up, `blow` of blow and a recovery of [`RECOVER`] wind-ups: loaded
/// builds in the wind-up and is released in the blow, thrown is the blow and then dies away over the
/// recovery.
pub fn phases(t: f32, wind: f32, blow: f32) -> (f32, f32) {
    let wind = wind.max(f32::EPSILON);
    let recover = (RECOVER * wind).max(f32::EPSILON);
    if t < wind {
        (ease(t / wind), 0.0)
    } else if t < wind + blow {
        let r = ease((t - wind) / blow.max(f32::EPSILON));
        (1.0 - r, r)
    } else {
        (0.0, 1.0 - ease((t - wind - blow) / recover))
    }
}

/// Returns the brace of an animal of body length `length` metres making `attack`: the hips drop by
/// [`CROUCH`] of the hip height times the commitment, the weight moves back by [`WEIGHT`] body lengths
/// times it, the body rears, ducks or turns its flank as the attack asks, and the same amounts run
/// the other way as the weight is thrown.
pub fn brace(attack: &Attacking, length: f32) -> Brace {
    let kind = attack.kind;
    let commit = COMMIT[kind as usize];
    let (load, thrown) = phases(attack.t, attack.wind, attack.blow());
    let (rear, duck, turn) = match kind {
        Kind::Trample => (REAR[0], 0.0, 0.0),
        Kind::Swipe => (REAR[1], 0.0, 0.0),
        Kind::Charge => (0.0, DUCK[0], 0.0),
        Kind::Lunge => (0.0, DUCK[1], 0.0),
        Kind::Sweep => (0.0, 0.0, FLANK),
        Kind::Bite => (0.0, 0.0, 0.0),
    };
    Brace {
        drop: CROUCH * commit * load,
        shift: WEIGHT * length * commit * (load - thrown),
        pitch: (rear - duck) * commit * load - OFF_BALANCE * commit * thrown,
        yaw: turn * (load - thrown),
    }
}

/// Sets the brace of `beast` for its `attack`, or at ease when it makes none.
pub fn hold(beast: &mut Beast, attack: Option<&Attacking>, length: f32) {
    let held = attack.map_or(Brace::default(), |a| brace(a, length));
    if beast.motion.brace != held {
        beast.motion.brace = held;
    }
}
