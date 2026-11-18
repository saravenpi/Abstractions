//! A planet's piece: every musical choice, drawn from the planet seed.

use super::mode::{Mode, Scale};
use super::mood::mood;
use crate::audio::dsp::rand::Rand;
use crate::universe::PlanetClass;

/// Lowest tonic, as a MIDI note in the bass register: C2, 65 Hz.
const LOW_TONIC: i32 = 36;

/// Tonics the home planet may use: C, D, F and G, the plainest keys.
const HOME_TONICS: [i32; 4] = [36, 38, 41, 43];

/// Distances in semitones from the bass to the middle upper voice: warm, open and high.
///
/// Anything closer than an eleventh puts thirds under 130 Hz, where they turn to mud.
const LIFTS: [i32; 3] = [17, 20, 24];

/// How the pad sounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timbre {
    /// Detune of the two sines of a voice, in cents either side.
    pub detune: f32,
    /// Level of the triangle partial, 0 for pure sines.
    pub triangle: f32,
    /// Level of the sine one octave up.
    pub shimmer: f32,
    /// Level of the colour voice (ninth or seventh), 0 for bare triads.
    pub colour: f32,
    /// Low-pass cutoff in hertz at midday.
    pub bright: f32,
}

/// The reverb of a piece.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// Level of the reverb against the dry pad.
    pub wet: f32,
    /// Scale of the delay lines: 1 is a hall, 1.8 a cathedral.
    pub size: f32,
    /// Seconds the tail takes to fall by 60 dB.
    pub tail: f32,
    /// Damping corner of the tail in hertz.
    pub dark: f32,
}

/// How time passes in a piece.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pace {
    /// Seconds each chord lasts.
    pub chord: f32,
    /// Seconds a chord takes to fade into the next.
    pub fade: f32,
    /// Mean seconds between two bell notes at midday.
    pub bell_gap: f32,
    /// Level of the bell notes.
    pub bell_level: f32,
}

/// Everything that makes one planet's music its own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    /// Planet seed the piece was drawn from.
    pub seed: u64,
    /// Whether this is the warm variant played on the home planet.
    pub home: bool,
    /// The planet class whose mood the piece took.
    pub class: PlanetClass,
    /// Tonic and mode.
    pub scale: Scale,
    /// Semitones from the bass to the middle upper voice.
    pub lift: i32,
    /// Pad sound.
    pub timbre: Timbre,
    /// Reverb.
    pub room: Room,
    /// Tempo of change.
    pub pace: Pace,
}

/// Draws a value in the `wild` range, or in the narrower `warm` range on the home planet.
fn draw(rand: &mut Rand, home: bool, wild: (f32, f32), warm: (f32, f32)) -> f32 {
    let (low, high) = if home { warm } else { wild };
    rand.range(low, high)
}

/// Draws the tonic and mode. The home planet is always in a plain major key.
fn scale(rand: &mut Rand, home: bool) -> Scale {
    let (tonic, mode) = (LOW_TONIC + rand.below(12) as i32, Mode::draw(rand));
    let plain = HOME_TONICS[rand.below(HOME_TONICS.len())];
    match home {
        true => Scale {
            tonic: plain,
            mode: Mode::Ionian,
        },
        false => Scale { tonic, mode },
    }
}

/// Draws the pad sound. A quarter of the wild pieces play bare triads, without the colour voice.
fn timbre(rand: &mut Rand, home: bool) -> Timbre {
    let bare = rand.unit() < 0.25 && !home;
    Timbre {
        detune: draw(rand, home, (3.0, 9.0), (4.0, 6.0)),
        triangle: draw(rand, home, (0.0, 0.45), (0.1, 0.2)),
        shimmer: draw(rand, home, (0.0, 0.3), (0.05, 0.12)),
        colour: draw(rand, home, (0.25, 0.6), (0.35, 0.5)) * f32::from(u8::from(!bare)),
        bright: draw(rand, home, (900.0, 3200.0), (1500.0, 2000.0)),
    }
}

/// Draws the reverb.
fn room(rand: &mut Rand, home: bool) -> Room {
    Room {
        wet: draw(rand, home, (0.3, 0.6), (0.4, 0.5)),
        size: draw(rand, home, (1.0, 1.8), (1.3, 1.5)),
        tail: draw(rand, home, (4.0, 9.0), (6.0, 7.0)),
        dark: draw(rand, home, (1800.0, 5000.0), (2600.0, 3200.0)),
    }
}

/// Draws the tempo of change.
fn pace(rand: &mut Rand, home: bool) -> Pace {
    Pace {
        chord: draw(rand, home, (12.0, 24.0), (16.0, 20.0)),
        fade: draw(rand, home, (4.0, 8.0), (6.0, 8.0)),
        bell_gap: draw(rand, home, (7.0, 22.0), (10.0, 14.0)),
        bell_level: draw(rand, home, (0.06, 0.14), (0.09, 0.11)),
    }
}

impl Piece {
    /// Draws the piece of an island world, the class of the home planet. `home` narrows every
    /// choice toward a warm, settled sound.
    pub fn new(seed: u64, home: bool) -> Self {
        let rand = &mut Rand::new(seed, 1);
        let scale = scale(rand, home);
        let lift = LIFTS[rand.below(LIFTS.len())];
        Self {
            seed,
            home,
            class: PlanetClass::Island,
            scale,
            lift: if home { 17 } else { lift },
            timbre: timbre(rand, home),
            room: room(rand, home),
            pace: pace(rand, home),
        }
    }

    /// Draws the piece of a planet of `class`: the piece of its seed, in the mode and with the
    /// brightness the class asks for. An island world keeps the piece exactly as drawn.
    pub fn of(seed: u64, home: bool, class: PlanetClass) -> Self {
        let (piece, mood) = (Self::new(seed, home), mood(class));
        let mode = mood.mode.unwrap_or(piece.scale.mode);
        let bright = piece.timbre.bright * mood.bright;
        Self {
            class,
            scale: Scale {
                mode,
                ..piece.scale
            },
            timbre: Timbre {
                bright,
                ..piece.timbre
            },
            ..piece
        }
    }
}
