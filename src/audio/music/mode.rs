//! Modes and scales: which notes a piece may use.

use crate::audio::dsp::rand::Rand;

/// The mode of a piece. All five are consonant: no locrian, no phrygian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Major with a raised fourth: bright and floating.
    Lydian,
    /// Plain major: warm and settled.
    Ionian,
    /// Minor with a raised sixth: cool but hopeful.
    Dorian,
    /// Natural minor: dark and calm.
    Aeolian,
    /// Major pentatonic: open, no semitone anywhere.
    Pentatonic,
}

/// Each mode with its weight out of 100 when a planet draws one.
const WEIGHTS: [(Mode, u32); 5] = [
    (Mode::Lydian, 26),
    (Mode::Ionian, 22),
    (Mode::Dorian, 20),
    (Mode::Aeolian, 16),
    (Mode::Pentatonic, 16),
];

impl Mode {
    /// Returns the semitones above the tonic of each scale degree.
    pub fn steps(self) -> &'static [i32] {
        match self {
            Mode::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Mode::Ionian => &[0, 2, 4, 5, 7, 9, 11],
            Mode::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Mode::Aeolian => &[0, 2, 3, 5, 7, 8, 10],
            Mode::Pentatonic => &[0, 2, 4, 7, 9],
        }
    }

    /// Draws a mode with the consonant weights.
    pub fn draw(rand: &mut Rand) -> Self {
        let mut roll = rand.below(100) as u32;
        for (mode, weight) in WEIGHTS {
            if roll < weight {
                return mode;
            }
            roll -= weight;
        }
        Mode::Ionian
    }
}

/// A mode placed on a tonic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    /// MIDI note of the tonic in the bass register.
    pub tonic: i32,
    /// The mode built on it.
    pub mode: Mode,
}

impl Scale {
    /// Returns the number of degrees in one octave.
    pub fn degrees(self) -> i32 {
        self.mode.steps().len() as i32
    }

    /// Returns the semitones above the tonic of `degree`, which may lie in any octave.
    pub fn offset(self, degree: i32) -> i32 {
        let len = self.degrees();
        self.mode.steps()[degree.rem_euclid(len) as usize] + 12 * degree.div_euclid(len)
    }

    /// Returns the pitch class, 0 to 11, of `degree`.
    pub fn class(self, degree: i32) -> i32 {
        (self.tonic + self.offset(degree)).rem_euclid(12)
    }

    /// Returns true when the chord stacked on `degree` is consonant.
    ///
    /// Seven-note modes need a perfect fifth above the root, which rules out their one diminished
    /// triad. Every pentatonic stack is consonant.
    pub fn consonant(self, degree: i32) -> bool {
        self.degrees() == 5 || self.offset(degree + 4) - self.offset(degree) == 7
    }

    /// Returns the degree of the colour tone above `degree`: the ninth over a major chord and
    /// over any pentatonic stack, the seventh over a minor chord.
    ///
    /// Either way the colour tone sits a whole tone from its neighbours, never a semitone.
    pub fn colour(self, degree: i32) -> i32 {
        let major = self.offset(degree + 2) - self.offset(degree) == 4;
        if major || self.degrees() == 5 {
            degree + 1
        } else {
            degree + self.degrees() - 1
        }
    }
}
