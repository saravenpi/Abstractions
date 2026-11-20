//! The rhythm of a call: how long its notes and the gaps between them last, which the sound and the
//! body that calls both follow, so the mouth opens on every note.
//!
//! Bigger animals call slower. Physiological times such as a breath or a heartbeat grow as the mass
//! to the power 1/4 (Lindstedt and Calder 1981, "Body size, physiological time, and longevity of
//! homeothermic animals", The Quarterly Review of Biology 56, 1-16, <https://doi.org/10.1086/412080>).
//! With the pitch falling as the mass to the power -0.4, a note lasts as the pitch to the power
//! -0.625.

use super::calls::Cry;
use super::voice::{Timbre, Voice};

/// Frequency in hertz at which notes last their base length.
const REF_HZ: f32 = 250.0;

/// Exponent of the pitch in the length of a note.
const LENGTH_EXPONENT: f32 = -0.625;

/// Shortest and longest factor of size on the length of a note.
const SIZE_SPAN: (f32, f32) = (0.35, 3.0);

/// Seconds of silence between two notes of a contact call at [`REF_HZ`].
const GAP: f32 = 0.08;

/// Most notes of one call.
pub const MOST_NOTES: u8 = 3;

/// The timing of one call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rhythm {
    /// Seconds one note lasts.
    pub note: f32,
    /// Seconds of silence between two notes.
    pub gap: f32,
    /// Notes in the call, 1 to [`MOST_NOTES`].
    pub notes: u8,
}

/// Returns the seconds a note of `timbre` lasts at [`REF_HZ`] in a contact call.
fn base(timbre: Timbre) -> f32 {
    match timbre {
        Timbre::Low => 0.55,
        Timbre::Hoot => 0.22,
        Timbre::Whistle => 0.14,
        Timbre::Growl => 0.6,
        Timbre::Hiss => 0.4,
        Timbre::Croak => 0.12,
        Timbre::Click => 0.02,
        Timbre::Rasp => 0.07,
    }
}

/// Returns the length factor and the gap factor of a call for `cry`: an alarm is short and hurried,
/// a threat long and slow.
fn shape(cry: Cry) -> (f32, f32) {
    match cry {
        Cry::Contact => (1.0, 1.0),
        Cry::Alarm => (0.55, 0.6),
        Cry::Chase => (1.5, 1.2),
        Cry::Distress => (1.3, 0.7),
        Cry::Threat => (1.8, 1.4),
        Cry::Young => (0.6, 0.8),
    }
}

/// Returns the rhythm of a call of `voice` for `cry`: slower for lower, bigger voices.
pub fn rhythm(voice: &Voice, cry: Cry) -> Rhythm {
    let size = (voice.hz.max(1.0) / REF_HZ)
        .powf(LENGTH_EXPONENT)
        .clamp(SIZE_SPAN.0, SIZE_SPAN.1);
    let (stretch, pace) = shape(cry);
    let notes = match cry {
        Cry::Distress => voice.notes + 1,
        Cry::Threat => 1,
        _ => voice.notes,
    };
    Rhythm {
        note: base(voice.timbre) * size * stretch,
        gap: GAP * size * pace,
        notes: notes.clamp(1, MOST_NOTES),
    }
}

impl Rhythm {
    /// Returns the seconds from the start of note `n` to the start of the next.
    pub fn period(&self) -> f32 {
        self.note + self.gap
    }

    /// Returns the seconds the whole call lasts.
    pub fn seconds(&self) -> f32 {
        f32::from(self.notes) * self.note + f32::from(self.notes.saturating_sub(1)) * self.gap
    }
}
