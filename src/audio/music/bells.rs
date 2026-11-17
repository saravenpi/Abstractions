//! Sparse high notes over the pad: soft, glassy, long to fade.

use super::voicing::Chord;
use crate::audio::dsp::osc::{Phasor, note_hz, sine};
use crate::audio::dsp::rand::Rand;

/// Bell notes that can ring at once.
const RINGING: usize = 4;

/// Lowest bell note, as a MIDI note: C5.
const LOW_NOTE: i32 = 72;

/// Note under which a bell may be struck one octave higher, as a MIDI note: G5.
const LIFT_UNDER: i32 = 79;

/// Seconds a bell takes to reach its level, long enough to have no click.
const ATTACK_SECONDS: f32 = 0.02;

/// Shortest and longest time in seconds for a bell to fall to a third of its level.
const DECAY_SECONDS: (f32, f32) = (1.0, 1.7);

/// Level of the partial a twelfth above the note.
const OVERTONE: f32 = 0.2;

/// Level under which a ringing bell may be struck again.
const QUIET: f32 = 1e-3;

/// One ringing note.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Bell {
    /// Fundamental.
    base: Phasor,
    /// Partial a twelfth above.
    over: Phasor,
    /// Turns per sample of the fundamental.
    step: f32,
    /// Current level.
    amp: f32,
    /// Factor applied to the level every sample.
    decay: f32,
    /// Attack position from 0 to 1.
    rise: f32,
    /// Stereo position from 0 (left) to 1 (right).
    pan: f32,
}

impl Bell {
    /// Renders one stereo frame. `attack` is the attack speed per sample.
    fn frame(&mut self, attack: f32) -> [f32; 2] {
        if self.amp < 1e-5 {
            return [0.0; 2];
        }
        self.rise = (self.rise + attack).min(1.0);
        self.amp *= self.decay;
        let tone =
            sine(self.base.tick(self.step)) + OVERTONE * sine(self.over.tick(self.step * 3.0));
        let out = tone * self.amp * self.rise * self.rise;
        [out * (1.0 - self.pan), out * self.pan]
    }
}

/// The bell player of a piece.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bells {
    /// The ringing notes.
    bells: [Bell; RINGING],
    /// Bell struck next.
    next: usize,
    /// Samples until the next note.
    wait: f32,
    /// Source of every random choice, seeded by the planet.
    rand: Rand,
    /// Sample rate in hertz.
    rate: f32,
}

impl Bells {
    /// Builds a silent player. The first note comes after about one `gap` in seconds.
    pub fn new(rate: f32, seed: u64, gap: f32) -> Self {
        let mut rand = Rand::new(seed, 3);
        Self {
            bells: [Bell::default(); RINGING],
            next: 0,
            wait: gap * rate * rand.range(0.7, 1.3),
            rand,
            rate,
        }
    }

    /// Strikes a note of `chord`, moved up into the bell register, at `level`.
    fn strike(&mut self, chord: &Chord, level: f32) {
        let bell = &mut self.bells[self.next];
        if bell.amp > QUIET {
            return;
        }
        self.next = (self.next + 1) % RINGING;
        let picked = chord.notes[1 + self.rand.below(4)];
        let low = picked + 12 * (LOW_NOTE - picked + 11).div_euclid(12).max(0);
        let note = low + 12 * (self.rand.below(2) as i32) * i32::from(low < LIFT_UNDER);
        let seconds = self.rand.range(DECAY_SECONDS.0, DECAY_SECONDS.1);
        *bell = Bell {
            step: note_hz(note as f32) / self.rate,
            amp: level * self.rand.range(0.6, 1.0),
            decay: (-1.0 / (seconds * self.rate)).exp(),
            pan: self.rand.range(0.2, 0.8),
            ..Bell::default()
        };
    }

    /// Renders one stereo frame. `gap` is the mean time in seconds between notes.
    pub fn frame(&mut self, chord: &Chord, gap: f32, level: f32) -> [f32; 2] {
        self.wait -= 1.0;
        if self.wait <= 0.0 {
            self.strike(chord, level);
            self.wait = gap * self.rate * self.rand.range(0.45, 1.6);
        }
        let attack = 1.0 / (ATTACK_SECONDS * self.rate);
        let mut out = [0.0; 2];
        for bell in &mut self.bells {
            let ring = bell.frame(attack);
            out = [out[0] + ring[0], out[1] + ring[1]];
        }
        out
    }
}
