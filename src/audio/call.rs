//! The call of an animal as a one-shot cue: its voice, why it calls and where it is for the listener.

use crate::life::signs::calls::{CRIES, Cry, TIMBRES, Voice};

/// Lowest frequency in hertz a call can carry; steps go up from it in eighths of an octave.
const BASE_HZ: f32 = 40.0;

/// Pitch steps per octave in a packed call.
const STEPS_PER_OCTAVE: f32 = 8.0;

/// Levels of the side and of the nearness in a packed call.
const LEVELS: f32 = 15.0;

/// One animal call, positioned for the listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Call {
    /// The voice of the species.
    pub voice: Voice,
    /// Why it calls.
    pub cry: Cry,
    /// Where it is: -1 at the listener's left to 1 at the right.
    pub side: f32,
    /// How loud it arrives, 0 out of earshot to 1 beside the head.
    pub near: f32,
}

impl Call {
    /// Packs the call into 23 bits: pitch 6, timbre 3, notes 2, cry 3, side 4, nearness 4, muffled 1.
    /// The pitch is rounded to an eighth of an octave, the side and nearness to fifteenths.
    pub(super) fn pack(self) -> u32 {
        let step = (STEPS_PER_OCTAVE * (self.voice.hz / BASE_HZ).max(1.0).log2()).round() as u32;
        let timbre = TIMBRES
            .iter()
            .position(|t| *t == self.voice.timbre)
            .unwrap_or(0) as u32;
        let notes = u32::from(self.voice.notes.clamp(1, 4) - 1);
        let cry = CRIES.iter().position(|c| *c == self.cry).unwrap_or(0) as u32;
        let side = ((self.side.clamp(-1.0, 1.0) + 1.0) * 0.5 * LEVELS).round() as u32;
        let near = (self.near.clamp(0.0, 1.0) * LEVELS).round() as u32;
        let muffled = u32::from(self.voice.muffled);
        step.min(63)
            | timbre << 6
            | notes << 9
            | cry << 11
            | side << 14
            | near << 18
            | muffled << 22
    }

    /// Unpacks a call packed by [`Call::pack`], or nothing when the bits hold none.
    pub(super) fn unpack(bits: u32) -> Option<Self> {
        let field = |shift: u32, width: u32| ((bits >> shift) & ((1 << width) - 1)) as usize;
        let voice = Voice {
            hz: BASE_HZ * (field(0, 6) as f32 / STEPS_PER_OCTAVE).exp2(),
            timbre: *TIMBRES.get(field(6, 3))?,
            notes: field(9, 2) as u8 + 1,
            muffled: field(22, 1) == 1,
        };
        Some(Self {
            voice,
            cry: *CRIES.get(field(11, 3))?,
            side: field(14, 4) as f32 / LEVELS * 2.0 - 1.0,
            near: field(18, 4) as f32 / LEVELS,
        })
    }
}
