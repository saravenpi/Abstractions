//! The recipe of one call, worked out once when it starts: the organ, the pitch, the notes, the
//! formants of the tract, the colour of the voice print and the way to the ear.
//!
//! Every species sounds its own: the voice print picks its vowel, its vibrato, whether it trills or
//! biphonates, the melody of a song. Every animal sounds a little its own: its byte in the herd moves
//! its pitch and its pace.

use super::notes::{NOTES, Note, notes};
use super::shape::{Source, lowered, mood, organ};
use super::tract::air;
use super::tune::{LEVEL, MUFFLE};
use crate::audio::call::Call;
use crate::audio::dsp::rand::Rand;
use crate::life::signs::calls::{Cry, Timbre, Voice};

/// Share by which the formants of a young rise over those of its parents: a shorter tract.
const YOUNG_FORMANT: f32 = 1.4;

/// Share of its length a lowered larynx adds to the tract.
const LOWERED_FORMANT: f32 = 0.8;

/// Share by which an individual's pitch and pace may differ from those of its species, either way.
const ONE_SPREAD: (f32, f32) = (0.06, 0.05);

/// The recipe of one call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    /// The organ that makes it.
    pub source: Source,
    /// Pitch in hertz, before the melody of each note.
    pub hz: f32,
    /// The notes, the first [`Plan::count`] of them sounding.
    pub notes: [Note; NOTES],
    /// How many notes sound.
    pub count: usize,
    /// The three formants in hertz.
    pub formants: [f32; 3],
    /// Share of breath in the voice.
    pub breath: f32,
    /// Vibrato: its depth as a share of the pitch and its rate in hertz.
    pub vibrato: (f32, f32),
    /// Rate in hertz of the trill of a song, 0 for none.
    pub trill: f32,
    /// Ratio of the second tone of a biphonating voice, 0 for none.
    pub second: f32,
    /// Teeth a second of a file, and the resonance they ring.
    pub teeth: (f32, f32),
    /// Peak level.
    pub gain: f32,
    /// Corner in hertz of the low-pass of the air on the way.
    pub air: f32,
    /// True when heard through the surface of the water.
    pub muffled: bool,
}

/// Returns the factors of pitch and of pace of individual `one`.
fn individual(one: u8) -> (f32, f32) {
    let (a, b) = (f32::from(one & 15) / 15.0, f32::from(one >> 4) / 15.0);
    (
        1.0 + ONE_SPREAD.0 * (2.0 * a - 1.0),
        1.0 + ONE_SPREAD.1 * (2.0 * b - 1.0),
    )
}

/// Returns the three formants of `voice` calling `cry`: odd multiples of half the spacing, moved by the
/// vowel of its print, higher for a young, lower for a larynx lowered to roar.
fn formants(voice: &Voice, cry: Cry) -> [f32; 3] {
    let vowel = [
        0.85 + 0.1 * f32::from(voice.print & 3),
        0.8 + 0.15 * f32::from(voice.print >> 2 & 3),
        1.0,
    ];
    let reach = match cry {
        Cry::Young => YOUNG_FORMANT,
        c if lowered(c) => LOWERED_FORMANT,
        _ => 1.0,
    };
    let hoot = if voice.timbre == Timbre::Hoot {
        0.7
    } else {
        1.0
    };
    let spacing = voice.formant * reach;
    [
        0.5 * spacing * vowel[0],
        1.5 * spacing * vowel[1] * hoot,
        2.5 * spacing * vowel[2],
    ]
}

/// Returns the vibrato, the trill and the second tone a voice of `print` made by `source` has.
fn colour(print: u8, source: Source) -> ((f32, f32), f32, f32) {
    let vibrato = (
        0.006 * f32::from(print >> 6 & 3),
        4.0 + f32::from(print & 3),
    );
    let trill = match source == Source::Syrinx && print & 0x0C == 0 {
        true => 18.0 + 4.0 * f32::from(print & 3),
        false => 0.0,
    };
    let second = match source == Source::Larynx && print & 0x81 == 0x81 {
        true => 1.37 + 0.03 * f32::from(print >> 2 & 7),
        false => 0.0,
    };
    (vibrato, trill, second)
}

/// Returns the teeth a second of the file of `timbre` at `hz` and the resonance they ring: a cricket's
/// teeth at the rate of its tone, a crystal's slow ticks, or a beetle's buzz on a dull shell.
fn file(timbre: Timbre, print: u8, hz: f32) -> (f32, f32) {
    match (timbre, print & 1) {
        (Timbre::Click, 1) => (hz, 25.0),
        (Timbre::Click, _) => (40.0 + 0.01 * hz, 14.0),
        _ => ((0.25 * hz).clamp(80.0, 600.0), 3.0),
    }
}

/// Returns the recipe of `call`, each call a little different by `rand`.
pub fn plan(call: &Call, rand: &mut Rand) -> Plan {
    let voice = call.voice;
    let (organ, mood) = (organ(voice.timbre, voice.print), mood(call.cry));
    let (pitch, pace) = individual(call.one);
    let mut hz = voice.hz * mood.rise * pitch * rand.range(0.97, 1.03);
    if organ.source == Source::File {
        hz = (hz * 1.25).clamp(1500.0, 9000.0);
    }
    let (notes, count) = notes(call, &organ, &mood, pace);
    let (vibrato, trill, second) = colour(voice.print, organ.source);
    let muffle = if voice.muffled { MUFFLE.1 } else { 1.0 };
    Plan {
        source: organ.source,
        hz,
        notes,
        count,
        formants: formants(&voice, call.cry),
        breath: organ.breath,
        vibrato,
        trill,
        second,
        teeth: file(voice.timbre, voice.print, hz),
        gain: LEVEL * call.near.clamp(0.0, 1.0) * mood.gain * muffle,
        air: air(call.metres),
        muffled: voice.muffled,
    }
}

impl Plan {
    /// A call with no notes: what an idle voice holds.
    pub const SILENT: Plan = Plan {
        source: Source::Breath,
        hz: 100.0,
        notes: [Note::REST; NOTES],
        count: 0,
        formants: [500.0, 1500.0, 2500.0],
        breath: 0.0,
        vibrato: (0.0, 5.0),
        trill: 0.0,
        second: 0.0,
        teeth: (100.0, 3.0),
        gain: 0.0,
        air: 8000.0,
        muffled: false,
    };

    /// Returns the seconds from the start of the call to the end of its last note.
    pub fn seconds(&self) -> f32 {
        self.notes[..self.count]
            .iter()
            .fold(0.0, |end, note| end.max(note.at + note.len))
    }

    /// Returns the note sounding `t` seconds into the call, and how far into it, 0 to 1.
    pub fn at(&self, t: f32) -> Option<(Note, f32)> {
        self.notes[..self.count]
            .iter()
            .rev()
            .find(|note| t >= note.at && t < note.at + note.len)
            .map(|note| (*note, (t - note.at) / note.len.max(1e-4)))
    }
}
