//! What makes each call sound as it does: the organ that makes it, from the timbre of the species,
//! and the melody, roughness and level of each cry.
//!
//! Air-breathers push air through a larynx: a train of glottal pulses shaped by the tract (Taylor and
//! Reby 2010, "The contribution of source-filter theory to mammal vocal communication research",
//! Journal of Zoology 280, 221-236). A rough voice runs through the bifurcations of a vocal fold, from a
//! clean cycle to subharmonics, then to deterministic chaos (Tokuda 2018, "Non-linear dynamics in
//! mammalian voice production", <https://doi.org/10.1537/ase.171130>): calls of alarm, threat and
//! distress carry them, and a threat grows rougher as it winds up to the charge. Birds sing through a
//! syrinx with a tract tuned to the note, close to a pure, sweeping tone (Riede et al. 2006, PNAS 103,
//! 5543-5548). Stridulators strike the teeth of a file one by one, each stroke a pulse that falls about
//! a tenth in pitch (Montealegre-Z et al. 2011, J Exp Biol 214, 2105-2117).

use crate::life::signs::calls::{Cry, Timbre};

/// The organ that makes a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Vocal folds: glottal pulses through formants.
    Larynx,
    /// A syrinx: a pure tone that sweeps, through a tract tuned to it.
    Syrinx,
    /// A file and a scraper: pulses that ring a resonance of the shell.
    File,
    /// Breath alone: a hiss or a snort through the tract.
    Breath,
}

/// How a timbre is made.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Organ {
    /// The organ.
    pub source: Source,
    /// Share of the sound that is breath, 0 to 1.
    pub breath: f32,
    /// Roughness of the voice before its cry, 0 clean to 1 chaotic.
    pub rough: f32,
    /// Pitch factors of a note at its start, middle and end.
    pub pitch: [f32; 3],
}

/// The melody, roughness and level of a cry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mood {
    /// Pitch factor of the whole call.
    pub rise: f32,
    /// Pitch factors of a note at its start, middle and end.
    pub pitch: [f32; 3],
    /// Roughness added at the start and at the end of a note.
    pub rough: [f32; 2],
    /// Level against a contact call at full voice.
    pub gain: f32,
}

/// The four melodies of a syrinx, picked by the voice print: rising, falling, arched, level.
const SWEEPS: [[f32; 3]; 4] = [
    [0.8, 1.1, 1.35],
    [1.35, 1.0, 0.8],
    [0.9, 1.3, 0.95],
    [1.0, 1.02, 1.0],
];

/// Returns how `timbre` is made; `print` picks the melody of a syrinx.
pub fn organ(timbre: Timbre, print: u8) -> Organ {
    let make = |source, breath, rough, pitch| Organ {
        source,
        breath,
        rough,
        pitch,
    };
    let file = [1.0, 0.96, 0.88];
    match timbre {
        Timbre::Low => make(Source::Larynx, 0.25, 0.05, [1.05, 1.0, 0.8]),
        Timbre::Hoot => make(Source::Larynx, 0.12, 0.0, [1.0, 1.0, 0.96]),
        Timbre::Whistle => make(
            Source::Syrinx,
            0.03,
            0.0,
            SWEEPS[usize::from(print >> 4 & 3)],
        ),
        Timbre::Growl => make(Source::Larynx, 0.4, 0.35, [0.9, 1.0, 0.75]),
        Timbre::Hiss => make(Source::Breath, 1.0, 0.0, [1.0, 1.0, 0.9]),
        Timbre::Croak => make(Source::Larynx, 0.15, 0.3, [1.2, 1.0, 0.9]),
        Timbre::Click | Timbre::Rasp => make(Source::File, 0.0, 0.0, file),
    }
}

/// Returns the melody, roughness and level of `cry`.
///
/// A contact call is soft and clean, a young one high, an alarm short, high and hard. A threat starts
/// low and clean and grows rough to its end, the wind-up of a charge. Distress is high and broken, and
/// a death cry falls away into chaos.
pub fn mood(cry: Cry) -> Mood {
    let make = |rise, pitch, rough, gain| Mood {
        rise,
        pitch,
        rough,
        gain,
    };
    match cry {
        Cry::Contact => make(1.0, [1.0, 1.04, 0.94], [0.0, 0.0], 0.6),
        Cry::Young => make(1.7, [1.0, 1.12, 1.05], [0.0, 0.05], 0.45),
        Cry::Alarm => make(1.3, [1.1, 1.25, 0.9], [0.35, 0.2], 1.0),
        Cry::Chase => make(0.85, [0.95, 1.05, 0.85], [0.4, 0.5], 0.9),
        Cry::Threat => make(0.7, [0.8, 0.95, 0.9], [0.1, 0.9], 1.0),
        Cry::Distress => make(1.5, [1.2, 1.35, 0.8], [0.6, 0.85], 1.0),
        Cry::Death => make(1.2, [1.1, 0.9, 0.55], [0.5, 1.0], 0.85),
    }
}

/// Returns true when a call of `cry` cannot wait its turn: every cry but contact and young.
pub fn urgent(cry: Cry) -> bool {
    !matches!(cry, Cry::Contact | Cry::Young)
}

/// Returns true when a larynx lowers itself for `cry`, as a roaring stag does, lengthening its tract to
/// sound bigger (Fitch and Reby 2001, Proc R Soc B 268, 1669-1675).
pub fn lowered(cry: Cry) -> bool {
    matches!(cry, Cry::Threat | Cry::Chase)
}
