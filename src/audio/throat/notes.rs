//! The notes of a call: when each starts, how long it lasts, how its pitch and roughness move, and
//! the breaths around them.
//!
//! The voiced notes keep the rhythm of `crate::life::signs::calls::rhythm`, the one the calling body
//! moves its mouth to. A big breather snorts before a threat and blows before an alarm, as deer and
//! horses do, and a dying animal lets out a last breath after its cry.

use super::shape::{Mood, Organ, Source};
use crate::audio::call::Call;
use crate::life::signs::calls::{Cry, rhythm};
use crate::render::smoothstep;

/// Most notes of one call: three of its rhythm, a breath before and one after.
pub const NOTES: usize = 5;

/// Pitch in hertz under which a breather blows before its alarm: a big animal.
const BLOW_HZ: f32 = 180.0;

/// Share of the length of a note its snort lasts, and the shortest and longest snort in seconds.
const SNORT: (f32, f32, f32) = (0.3, 0.08, 0.35);

/// One note of a call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    /// Seconds from the start of the call.
    pub at: f32,
    /// Seconds it lasts.
    pub len: f32,
    /// Pitch factors at its start, middle and end.
    pub pitch: [f32; 3],
    /// Roughness at its start and end, 0 clean to 1 chaotic.
    pub rough: [f32; 2],
    /// False for a breath with no voice in it.
    pub voiced: bool,
    /// Level against the call.
    pub gain: f32,
}

impl Note {
    /// A silent note, the base every slot of a call starts from.
    pub const REST: Note = Note {
        at: 0.0,
        len: 0.0,
        pitch: [1.0; 3],
        rough: [0.0; 2],
        voiced: true,
        gain: 0.0,
    };

    /// Returns the pitch factor at `u`, from 0 at the start of the note to 1 at its end, easing
    /// through the middle.
    pub fn pitch(&self, u: f32) -> f32 {
        let [a, b, c] = self.pitch;
        match u < 0.5 {
            true => a + (b - a) * smoothstep(0.0, 0.5, u),
            false => b + (c - b) * smoothstep(0.5, 1.0, u),
        }
    }

    /// Returns the roughness at `u`.
    pub fn rough(&self, u: f32) -> f32 {
        (self.rough[0] + (self.rough[1] - self.rough[0]) * u).clamp(0.0, 1.0)
    }

    /// Returns the envelope at `u`: a smooth rise and fall that start and end on zero, with a body that
    /// sinks a little as the breath runs out, and quickly for a breath.
    pub fn envelope(&self, u: f32) -> f32 {
        let t = u * self.len;
        let attack = (0.15 * self.len).clamp(0.004, 0.06);
        let release = (0.25 * self.len).clamp(0.006, 0.12);
        let edges = smoothstep(0.0, attack, t) * smoothstep(0.0, release, self.len - t);
        let body = match self.voiced {
            true => 1.0 - 0.3 * u,
            false => (1.0 - u) * (1.0 - u),
        };
        self.gain * edges * body
    }
}

/// Returns a breath of `len` seconds at `at`, at `gain`.
fn breath(at: f32, len: f32, gain: f32) -> Note {
    Note {
        at,
        len,
        voiced: false,
        gain,
        ..Note::REST
    }
}

/// Returns the breath before `call`, if it has one: a snort before the threat of a breather, a blow
/// before the alarm of a big one.
fn lead(call: &Call, organ: &Organ, len: f32) -> Option<Note> {
    let breathes = matches!(organ.source, Source::Larynx | Source::Breath);
    let snorts = match call.cry {
        Cry::Threat => breathes,
        Cry::Alarm => breathes && call.voice.hz < BLOW_HZ,
        _ => false,
    };
    let snort = (SNORT.0 * len).clamp(SNORT.1, SNORT.2);
    snorts.then(|| breath(0.0, snort, 0.8))
}

/// Returns the notes of `call` made by `organ` in `mood`, the rhythm slowed by `pace`, and how many
/// there are.
pub fn notes(call: &Call, organ: &Organ, mood: &Mood, pace: f32) -> ([Note; NOTES], usize) {
    let beat = rhythm(&call.voice, call.cry);
    let (len, period) = (beat.note * pace, beat.period() * pace);
    let mut notes = [Note::REST; NOTES];
    let lead = lead(call, organ, len);
    let start = lead.map_or(0.0, |snort| snort.len);
    let mut count = usize::from(lead.is_some());
    notes[0] = lead.unwrap_or(Note::REST);
    let pitch = |i: usize, lift: f32| organ.pitch[i] * mood.pitch[i] * lift;
    for k in 0..usize::from(beat.notes).min(NOTES - 2) {
        let lift = 1.0 + 0.06 * (k % 2) as f32;
        notes[count] = Note {
            at: start + k as f32 * period,
            len,
            pitch: [pitch(0, lift), pitch(1, lift), pitch(2, lift)],
            rough: mood.rough.map(|r| organ.rough + r),
            voiced: organ.source != Source::Breath,
            gain: 1.0,
        };
        count += 1;
    }
    if call.cry == Cry::Death {
        let last = notes[count - 1];
        notes[count] = breath(last.at + last.len, 0.5 * last.len, 0.5);
        count += 1;
    }
    (notes, count)
}
