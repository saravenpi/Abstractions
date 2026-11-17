//! Voice leading: where each voice goes when the chord changes.

use super::mode::Scale;

/// Voices in a chord: the bass, three upper voices and the colour tone.
pub const VOICES: usize = 5;

/// Largest move of any voice between two chords, in semitones: a perfect fifth.
pub const LEAP: i32 = 7;

/// The six ways to hand three chord tones to three voices.
const ORDERS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

/// One chord of the progression, voiced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    /// Scale degree of the root.
    pub degree: i32,
    /// MIDI note of each voice: bass, three upper voices, colour tone.
    pub notes: [i32; VOICES],
}

/// Returns the pitch classes of the triad on `degree`: root, third and fifth.
pub fn triad(scale: Scale, degree: i32) -> [i32; 3] {
    [0, 2, 4].map(|step| scale.class(degree + step))
}

/// Returns the note of pitch class `class` that a voice at `prev` moves to.
///
/// It is the closer of the notes above and below. When both lie within `reach` semitones, the
/// one nearer to `home` wins, which is what keeps a voice in its register.
pub fn nearest(class: i32, prev: i32, home: i32, reach: i32) -> i32 {
    let up = (class - prev).rem_euclid(12);
    let down = up - 12;
    let both = up <= reach && -down <= reach;
    let rise = if both {
        (prev + up - home).abs() <= (prev + down - home).abs()
    } else {
        up <= 6
    };
    prev + if rise { up } else { down }
}

/// Returns how bad a voicing of the upper voices is: motion, distance from home, crossed voices.
fn cost(upper: [i32; 3], bass: i32, prev: &[i32; VOICES], homes: &[i32; VOICES]) -> i32 {
    let moved = |v: usize| 2 * (upper[v] - prev[v + 1]).abs() + (upper[v] - homes[v + 1]).abs();
    let crossed = [
        bass + 2 >= upper[0],
        upper[0] >= upper[1],
        upper[1] >= upper[2],
    ];
    let crossings = crossed.iter().filter(|&&c| c).count() as i32;
    moved(0) + moved(1) + moved(2) + 12 * crossings
}

/// Voices the chord on `degree` so that every voice moves as little as possible from `prev`.
///
/// No voice moves more than [`LEAP`] semitones: the bass takes the root, the upper voices share
/// the triad in the cheapest order, the colour voice takes the ninth or the seventh.
pub fn voice(scale: Scale, degree: i32, prev: &[i32; VOICES], homes: &[i32; VOICES]) -> Chord {
    let classes = triad(scale, degree);
    let bass = nearest(classes[0], prev[0], homes[0], LEAP);
    let place = |order: [usize; 3]| {
        [0, 1, 2].map(|v| nearest(classes[order[v]], prev[v + 1], homes[v + 1], 6))
    };
    let best = ORDERS
        .map(place)
        .into_iter()
        .min_by_key(|&upper| cost(upper, bass, prev, homes))
        .unwrap_or([prev[1], prev[2], prev[3]]);
    let colour = nearest(scale.class(scale.colour(degree)), prev[4], homes[4], 6);
    Chord {
        degree,
        notes: [bass, best[0], best[1], best[2], colour],
    }
}
