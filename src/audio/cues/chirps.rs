//! The voice of the robot companion: one short phrase of blips for each thing it says.

use super::sounds::tone;
use super::voice::{Blip, Wave};
use crate::audio::cue::ChirpKind;

/// Makes a blip slow to start, for the drowsy and the hummed notes.
const fn slow(blip: Blip) -> Blip {
    Blip {
        attack: 0.05,
        ..blip
    }
}

/// Makes a blip a pure, soft-edged note, for the hummed tune.
const fn hummed(blip: Blip) -> Blip {
    Blip {
        wave: Wave::Sine,
        attack: 0.03,
        ..blip
    }
}

/// Happy: three quick rising blips.
const CHIRP_HAPPY: [Blip; 3] = [
    tone(920.0, 1280.0, 0.07, 0.0, 0.13),
    tone(1180.0, 1660.0, 0.07, 0.085, 0.13),
    tone(1560.0, 2300.0, 0.1, 0.17, 0.13),
];

/// Curious: a short note, then a longer one gliding up like a question.
const CHIRP_CURIOUS: [Blip; 2] = [
    tone(1020.0, 1080.0, 0.08, 0.0, 0.13),
    tone(1080.0, 1900.0, 0.18, 0.12, 0.13),
];

/// Surprised: one fast blip that jumps up, and its small echo.
const CHIRP_SURPRISED: [Blip; 2] = [
    tone(900.0, 2400.0, 0.09, 0.0, 0.13),
    tone(2400.0, 2520.0, 0.05, 0.11, 0.07),
];

/// Sleepy: two slow, low, falling notes.
const CHIRP_SLEEPY: [Blip; 2] = [
    slow(tone(700.0, 430.0, 0.36, 0.0, 0.1)),
    slow(tone(460.0, 300.0, 0.42, 0.3, 0.08)),
];

/// Worried: three falling, hurried blips.
const CHIRP_WORRIED: [Blip; 3] = [
    tone(1500.0, 1120.0, 0.08, 0.0, 0.13),
    tone(1380.0, 1000.0, 0.08, 0.095, 0.13),
    tone(1250.0, 780.0, 0.14, 0.19, 0.13),
];

/// Delight: a bright trill up a major chord that ends on a short held note.
const CHIRP_DELIGHT: [Blip; 4] = [
    tone(1318.5, 1318.5, 0.05, 0.0, 0.11),
    tone(1568.0, 1568.0, 0.05, 0.055, 0.11),
    tone(1975.5, 1975.5, 0.05, 0.11, 0.11),
    tone(2637.0, 3136.0, 0.14, 0.165, 0.12),
];

/// Hum: the first three notes of a small tune, soft and unhurried.
const CHIRP_HUM: [Blip; 3] = [
    hummed(tone(523.3, 523.3, 0.18, 0.0, 0.09)),
    hummed(tone(659.3, 659.3, 0.18, 0.2, 0.09)),
    hummed(tone(784.0, 784.0, 0.26, 0.4, 0.09)),
];

/// Wake: a low blip, then a quick rise, like eyes snapping open.
const CHIRP_WAKE: [Blip; 2] = [
    tone(420.0, 400.0, 0.08, 0.0, 0.11),
    tone(600.0, 1500.0, 0.12, 0.11, 0.12),
];

/// Returns the blips of a companion chirp.
pub(super) fn chirp(kind: ChirpKind) -> &'static [Blip] {
    match kind {
        ChirpKind::Happy => &CHIRP_HAPPY,
        ChirpKind::Curious => &CHIRP_CURIOUS,
        ChirpKind::Surprised => &CHIRP_SURPRISED,
        ChirpKind::Sleepy => &CHIRP_SLEEPY,
        ChirpKind::Worried => &CHIRP_WORRIED,
        ChirpKind::Delight => &CHIRP_DELIGHT,
        ChirpKind::Hum => &CHIRP_HUM,
        ChirpKind::Wake => &CHIRP_WAKE,
    }
}
