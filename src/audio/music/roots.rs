//! The root progression: a closed loop of scale degrees that moves by thirds and fourths.

use super::mode::Scale;
use crate::audio::dsp::rand::Rand;

/// Most chords one loop can hold.
pub const MAX_LOOP: usize = 8;

/// Root moves allowed between two chords, in scale degrees: a third or a fourth, up or down.
///
/// Triads a third apart share two tones and triads a fourth apart share one, so every move keeps
/// a common tone.
const MOVES: [i32; 4] = [2, -2, 3, -3];

/// Returns true when the root may move from degree `from` to degree `to`.
pub fn related(scale: Scale, from: i32, to: i32) -> bool {
    MOVES
        .iter()
        .any(|step| (from + step - to).rem_euclid(scale.degrees()) == 0)
}

/// Draws the next root after `from`, a consonant chord one allowed move away.
///
/// Going straight back to `avoid` is redrawn once, so loops rarely rock between two chords.
fn step(scale: Scale, rand: &mut Rand, from: i32, avoid: i32) -> Option<i32> {
    let mut options = [0; 4];
    let mut count = 0;
    for to in MOVES.map(|step| (from + step).rem_euclid(scale.degrees())) {
        if scale.consonant(to) && !options[..count].contains(&to) {
            options[count] = to;
            count += 1;
        }
    }
    let draw = |rand: &mut Rand| options[rand.below(count.max(1))];
    let first = draw(rand);
    let pick = if first == avoid { draw(rand) } else { first };
    (count > 0).then_some(pick)
}

/// Draws a loop of `len` roots that starts on the tonic, or nothing when the walk does not close.
pub fn walk(scale: Scale, rand: &mut Rand, len: usize) -> Option<[i32; MAX_LOOP]> {
    let mut roots = [0; MAX_LOOP];
    for i in 1..len {
        let avoid = if i >= 2 { roots[i - 2] } else { -1 };
        roots[i] = step(scale, rand, roots[i - 1], avoid)?;
    }
    related(scale, roots[len - 1], roots[0]).then_some(roots)
}
