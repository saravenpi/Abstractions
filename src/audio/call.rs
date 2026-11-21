//! The call of an animal as a one-shot cue: its voice, why it calls, who calls and where it is for
//! the listener.

use crate::life::signs::calls::{CRIES, Cry, TIMBRES, Voice};

/// Lowest frequency in hertz a call can carry; steps go up from it in sixteenths of an octave.
const BASE_HZ: f32 = 40.0;

/// Pitch steps per octave in a packed call.
const STEPS_PER_OCTAVE: f32 = 16.0;

/// Lowest spacing of the formants in hertz; steps go up from it in eighths of an octave.
const BASE_FORMANT: f32 = 50.0;

/// Formant steps per octave in a packed call.
const FORMANT_STEPS: f32 = 8.0;

/// Steps per octave of the distance in a packed call, from one metre.
const METRE_STEPS: f32 = 3.0;

/// Levels of the side and of the nearness in a packed call.
const LEVELS: f32 = 15.0;

/// One animal call, positioned for the listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Call {
    /// The voice of the species.
    pub voice: Voice,
    /// Why it calls.
    pub cry: Cry,
    /// Who calls in its herd, a byte that shifts its pitch and its pace a little.
    pub one: u8,
    /// Where it is: -1 at the listener's left to 1 at the right.
    pub side: f32,
    /// How loud it arrives, 0 out of earshot to 1 beside the head.
    pub near: f32,
    /// How far it is in metres, which takes the highs the air absorbs on the way.
    pub metres: f32,
}

/// Returns `value` as the whole number of `steps` per octave over `base`, at most `top`.
fn octaves(value: f32, base: f32, steps: f32, top: u64) -> u64 {
    ((steps * (value / base).max(1.0).log2()).round() as u64).min(top)
}

impl Call {
    /// Packs the call into 51 bits: pitch 7, timbre 3, notes 2, cry 3, muffled 1, formant 6, print 8,
    /// individual 8, then side 4, nearness 4 and distance 5. The pitch is rounded to a sixteenth of an
    /// octave, the formants to an eighth, the distance to a third, the side and nearness to fifteenths.
    pub(super) fn pack(self) -> u64 {
        let v = self.voice;
        let timbre = TIMBRES.iter().position(|t| *t == v.timbre).unwrap_or(0) as u64;
        let cry = CRIES.iter().position(|c| *c == self.cry).unwrap_or(0) as u64;
        octaves(v.hz, BASE_HZ, STEPS_PER_OCTAVE, 127)
            | timbre << 7
            | u64::from(v.notes.clamp(1, 4) - 1) << 10
            | cry << 12
            | u64::from(v.muffled) << 15
            | octaves(v.formant, BASE_FORMANT, FORMANT_STEPS, 63) << 16
            | u64::from(v.print) << 22
            | u64::from(self.one) << 30
            | self.place() << 38
    }

    /// Packs where the call is: side 4 bits, nearness 4 and distance 5.
    fn place(self) -> u64 {
        let side = ((self.side.clamp(-1.0, 1.0) + 1.0) * 0.5 * LEVELS).round() as u64;
        let near = (self.near.clamp(0.0, 1.0) * LEVELS).round() as u64;
        side | near << 4 | octaves(self.metres, 1.0, METRE_STEPS, 31) << 8
    }

    /// Unpacks a call packed by [`Call::pack`], or nothing when the bits hold none.
    pub(super) fn unpack(bits: u64) -> Option<Self> {
        let field = |shift: u32, width: u32| ((bits >> shift) & ((1 << width) - 1)) as usize;
        let voice = Voice {
            hz: BASE_HZ * (field(0, 7) as f32 / STEPS_PER_OCTAVE).exp2(),
            timbre: *TIMBRES.get(field(7, 3))?,
            notes: field(10, 2) as u8 + 1,
            muffled: field(15, 1) == 1,
            formant: BASE_FORMANT * (field(16, 6) as f32 / FORMANT_STEPS).exp2(),
            print: field(22, 8) as u8,
        };
        Some(Self {
            voice,
            cry: *CRIES.get(field(12, 3))?,
            one: field(30, 8) as u8,
            side: field(38, 4) as f32 / LEVELS * 2.0 - 1.0,
            near: field(42, 4) as f32 / LEVELS,
            metres: (field(46, 5) as f32 / METRE_STEPS).exp2(),
        })
    }
}
