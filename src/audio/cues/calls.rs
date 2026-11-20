//! The calls of animals: a few blips shaped by the voice of the species and why it calls.
//!
//! Every timbre is one blip per note, plus a breath or a rasp of noise for the voiced ones. An alarm
//! is higher, shorter and hurried; a hunter on the chase is lower and longer.

use super::sounds::{noise, tone};
use super::voice::{Blip, Wave};
use crate::audio::call::Call;
use crate::audio::dsp::rand::Rand;
use crate::life::signs::calls::{Cry, Timbre};

/// Most blips of one call.
pub const CALL_BLIPS: usize = 4;

/// Level of a call heard beside the head.
const LEVEL: f32 = 0.14;

/// Returns the pitch factor, length factor and seconds between notes of a call for `cry`.
fn shape(cry: Cry) -> (f32, f32, f32) {
    match cry {
        Cry::Contact => (1.0, 1.0, 1.0),
        Cry::Alarm => (1.3, 0.55, 0.6),
        Cry::Chase => (0.8, 1.5, 1.2),
    }
}

/// Returns one note of a call of `timbre` at `hz` hertz, `len` seconds long, starting after `delay`.
fn note(timbre: Timbre, hz: f32, len: f32, delay: f32) -> Blip {
    let pure = |wave, from: f32, to: f32, seconds: f32, attack| Blip {
        wave,
        attack,
        ..tone(from, to, seconds, delay, LEVEL)
    };
    match timbre {
        Timbre::Low => pure(Wave::Soft, hz * 1.05, hz * 0.8, 0.55 * len, 0.06),
        Timbre::Hoot => pure(Wave::Sine, hz, hz * 0.96, 0.22 * len, 0.03),
        Timbre::Whistle => pure(Wave::Sine, hz * 2.0, hz * 2.7, 0.14 * len, 0.01),
        Timbre::Growl => pure(Wave::Soft, hz * 0.9, hz * 0.7, 0.6 * len, 0.04),
        Timbre::Hiss => noise(
            (hz * 8.0).clamp(2500.0, 8000.0),
            0.4 * len,
            0.06,
            delay,
            LEVEL,
        ),
        Timbre::Croak => pure(Wave::Soft, hz * 1.2, hz * 0.9, 0.12 * len, 0.005),
        Timbre::Click => noise(
            (hz * 4.0).clamp(800.0, 7000.0),
            0.02 * len,
            0.001,
            delay,
            LEVEL,
        ),
    }
}

/// Returns the noise that rides on a note of `timbre` at `hz` hertz, `len` seconds long, if any:
/// the breath of a moan, the rasp of a growl or a croak.
fn grit(timbre: Timbre, hz: f32, len: f32) -> Option<Blip> {
    match timbre {
        Timbre::Low => Some(noise(hz * 3.0, 0.4 * len, 0.05, 0.02, 0.4 * LEVEL)),
        Timbre::Growl => Some(noise(hz * 2.5, 0.6 * len, 0.03, 0.0, 0.8 * LEVEL)),
        Timbre::Croak => Some(noise(hz * 4.0, 0.1 * len, 0.004, 0.0, 0.5 * LEVEL)),
        _ => None,
    }
}

/// Returns the blips of `call` and how many of them sound, each call a little different.
pub fn call(call: &Call, rand: &mut Rand) -> ([Blip; CALL_BLIPS], usize) {
    let (rise, stretch, pace) = shape(call.cry);
    let hz = call.voice.hz * rise * rand.range(0.95, 1.05);
    let len = stretch * rand.range(0.9, 1.1);
    let mut blips = [Blip::BASE; CALL_BLIPS];
    let mut count = 0;
    let grit = grit(call.voice.timbre, hz, len);
    let notes = usize::from(call.voice.notes).min(CALL_BLIPS - usize::from(grit.is_some()));
    let gap = note(call.voice.timbre, hz, len, 0.0).seconds + 0.08 * pace;
    for n in 0..notes {
        let lift = 1.0 + 0.06 * (n % 2) as f32;
        blips[count] = note(call.voice.timbre, hz * lift, len, gap * n as f32);
        count += 1;
    }
    if let Some(grit) = grit {
        blips[count] = grit;
        count += 1;
    }
    (blips, count)
}
