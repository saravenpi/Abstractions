//! The body in the wind-up: before an attack the whole body prepares, not only the part that makes
//! it, and after it the body is off balance.
//!
//! The hips drop by a share of the hip height that grows with the commitment of the attack, the weight
//! moves back, a trample or a downward swipe rears, a charge or a lunge ducks its head end, a tail
//! sweep turns the flank. The blow throws the weight forward and the recovery shows it off balance
//! for as long as the animal is open, then everything is back at ease. The part that strikes turns on
//! its joints: the neck bends back and throws the head out, a forefoot lifts and strikes, the tail
//! cocks and sweeps.

use super::kinds::Kind;
use super::timing::Attacking;
use super::tune::{
    COMMIT, CROUCH, DRAW_BACK, DUCK, FLANK, HEAD_DOWN, LASH, OFF_BALANCE, PAW, REAR, RECOVER,
    STAMP, WEIGHT,
};
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
        ..striker(kind, (load, thrown))
    }
}

/// Returns the brace of the part that makes an attack of `kind`, `load` wound up and `thrown` out,
/// each 0 to 1, the body itself at ease: a bite or a lunge draws the neck back and opens the jaw, then
/// throws the head out along the neck's reach; a charge drops the head and drives the horns ahead; a
/// swipe lifts a forefoot and strikes ahead with it; a trample lifts the forefeet and stamps them
/// down; a tail sweep cocks the tail and sweeps it through. Joints turn, lengths stay.
fn striker(kind: Kind, (load, thrown): (f32, f32)) -> Brace {
    let at_ease = Brace::default();
    match kind {
        Kind::Bite | Kind::Lunge => Brace {
            neck: DRAW_BACK * load,
            reach: thrown,
            gape: load,
            ..at_ease
        },
        Kind::Charge => Brace {
            neck: -HEAD_DOWN * load.max(thrown),
            reach: thrown,
            ..at_ease
        },
        Kind::Swipe => Brace {
            lift: PAW[0] * (load + 0.5 * thrown).min(1.0),
            claw: PAW[1] * thrown,
            ..at_ease
        },
        Kind::Trample => Brace {
            lift: STAMP[0] * load,
            claw: STAMP[1] * thrown,
            ..at_ease
        },
        Kind::Sweep => Brace {
            lash: LASH * (load - thrown),
            ..at_ease
        },
    }
}

/// Sets the brace of `beast` for its `attack`, or at ease when it makes none.
pub fn hold(beast: &mut Beast, attack: Option<&Attacking>, length: f32) {
    let held = attack.map_or(Brace::default(), |a| brace(a, length));
    if beast.motion.brace != held {
        beast.motion.brace = held;
    }
}
