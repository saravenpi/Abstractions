//! The progression of a piece: a loop of roots, voiced one chord at a time.

use super::mode::Scale;
use super::roots::{MAX_LOOP, walk};
use super::voicing::{Chord, VOICES, voice};
use crate::audio::dsp::rand::Rand;

/// Root loops tried before giving up and holding the tonic.
const TRIES: usize = 256;

/// Farthest a voice may sit from its home note, in semitones, for a loop to be kept.
pub const SPAN: i32 = 9;

/// Times a loop is played through when checking that its voices stay in register.
const ROUNDS: usize = 8;

/// The chord sequence of one piece. It never ends and never allocates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Harmony {
    /// Scale the chords are built from.
    scale: Scale,
    /// Root degree of each chord of the loop.
    roots: [i32; MAX_LOOP],
    /// Chords in the loop.
    len: usize,
    /// Index in the loop of the current chord.
    at: usize,
    /// Note each voice is pulled toward.
    homes: [i32; VOICES],
    /// The chord sounding now.
    chord: Chord,
}

impl Harmony {
    /// Composes the progression for a scale. `lift` is the distance in semitones from the bass
    /// to the middle upper voice. The same arguments always give the same progression.
    pub fn new(scale: Scale, lift: i32, seed: u64) -> Self {
        let mut rand = Rand::new(seed, 2);
        for _ in 0..TRIES {
            let len = 4 + rand.below(MAX_LOOP - 3);
            let found =
                walk(scale, &mut rand, len).map(|roots| Self::start(scale, lift, roots, len));
            if let Some(harmony) = found.filter(Self::steady) {
                return harmony;
            }
        }
        Self::start(scale, lift, [0; MAX_LOOP], 1)
    }

    /// Builds the progression of a root loop, standing on its first chord.
    fn start(scale: Scale, lift: i32, roots: [i32; MAX_LOOP], len: usize) -> Self {
        let mid = scale.tonic + lift;
        let homes = [scale.tonic, mid - 5, mid, mid + 5, mid + 12];
        let chord = voice(scale, roots[0], &homes, &homes);
        Self {
            scale,
            roots,
            len,
            at: 0,
            homes,
            chord,
        }
    }

    /// Returns true when every voice stays within [`SPAN`] of its home over many rounds.
    fn steady(&self) -> bool {
        let mut probe = *self;
        let near =
            |chord: Chord| (0..VOICES).all(|v| (chord.notes[v] - self.homes[v]).abs() <= SPAN);
        (0..ROUNDS * self.len).all(|_| near(probe.advance()))
    }

    /// Returns the chord sounding now.
    pub fn chord(&self) -> Chord {
        self.chord
    }

    /// Returns the scale of the piece.
    #[cfg(test)]
    pub fn scale(&self) -> Scale {
        self.scale
    }

    /// Returns the number of chords in the loop.
    #[cfg(test)]
    pub fn chords(&self) -> usize {
        self.len
    }

    /// Returns the note each voice is pulled toward.
    #[cfg(test)]
    pub fn homes(&self) -> [i32; VOICES] {
        self.homes
    }

    /// Moves to the next chord of the loop and returns it.
    pub fn advance(&mut self) -> Chord {
        self.at = (self.at + 1) % self.len;
        self.chord = voice(
            self.scale,
            self.roots[self.at],
            &self.chord.notes,
            &self.homes,
        );
        self.chord
    }
}
