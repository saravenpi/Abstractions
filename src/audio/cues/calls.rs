//! The calls of animals: a few blips shaped by the voice of the species and why it calls.
//!
//! Every timbre is one blip per note, plus a breath or a rasp of noise for the voiced ones. The notes
//! keep the rhythm of `crate::life::signs::calls::rhythm`, the one the calling body moves to. An
//! alarm is higher, a hunter on the chase or a threat lower, a young or a hurt animal higher still.
//! A muffled call, from under water, loses its highs and half its level.

use super::sounds::{noise, tone};
use super::voice::{Blip, Wave};
use crate::audio::call::Call;
use crate::audio::dsp::rand::Rand;
use crate::life::signs::calls::{Cry, Timbre, rhythm};

/// Most blips of one call.
pub const CALL_BLIPS: usize = 4;

/// Level of a call heard beside the head.
const LEVEL: f32 = 0.14;

/// Share of the frequency of its noise a muffled call keeps.
const MUFFLE_HZ: f32 = 0.35;

/// Share of the level a muffled call keeps.
const MUFFLE_LEVEL: f32 = 0.5;

/// Returns the pitch factor of a call for `cry`.
fn rise(cry: Cry) -> f32 {
    match cry {
        Cry::Contact => 1.0,
        Cry::Alarm => 1.3,
        Cry::Chase => 0.8,
        Cry::Distress => 1.5,
        Cry::Threat => 0.7,
        Cry::Young => 1.7,
    }
}

/// Returns one note of a call of `timbre` at `hz` hertz, `len` seconds long, starting after `delay`.
fn note(timbre: Timbre, hz: f32, len: f32, delay: f32) -> Blip {
    let pure = |wave, from: f32, to: f32, attack: f32| Blip {
        wave,
        attack: attack.min(0.3 * len),
        ..tone(from, to, len, delay, LEVEL)
    };
    let burst = |hz: f32, attack: f32| noise(hz, len, attack.min(0.3 * len), delay, LEVEL);
    match timbre {
        Timbre::Low => pure(Wave::Soft, hz * 1.05, hz * 0.8, 0.06),
        Timbre::Hoot => pure(Wave::Sine, hz, hz * 0.96, 0.03),
        Timbre::Whistle => pure(Wave::Sine, hz * 2.0, hz * 2.7, 0.01),
        Timbre::Growl => pure(Wave::Soft, hz * 0.9, hz * 0.7, 0.04),
        Timbre::Hiss => burst((hz * 8.0).clamp(2500.0, 8000.0), 0.06),
        Timbre::Croak => pure(Wave::Soft, hz * 1.2, hz * 0.9, 0.005),
        Timbre::Click => burst((hz * 4.0).clamp(800.0, 7000.0), 0.001),
        Timbre::Rasp => burst((hz * 3.0).clamp(600.0, 5000.0), 0.002),
    }
}

/// Returns the noise that rides on a note of `timbre` at `hz` hertz, `len` seconds long, if any:
/// the breath of a moan, the rasp of a growl or a croak.
fn grit(timbre: Timbre, hz: f32, len: f32) -> Option<Blip> {
    match timbre {
        Timbre::Low => Some(noise(
            hz * 3.0,
            0.75 * len,
            0.05_f32.min(0.2 * len),
            0.0,
            0.4 * LEVEL,
        )),
        Timbre::Growl => Some(noise(
            hz * 2.5,
            len,
            0.03_f32.min(0.2 * len),
            0.0,
            0.8 * LEVEL,
        )),
        Timbre::Croak => Some(noise(hz * 4.0, 0.8 * len, 0.004, 0.0, 0.5 * LEVEL)),
        _ => None,
    }
}

/// Returns `blip` as heard through the surface of the water: lower and quieter.
fn muffle(blip: Blip) -> Blip {
    let wave = match blip.wave {
        Wave::Noise => Wave::Noise,
        _ => Wave::Sine,
    };
    let from = match blip.wave {
        Wave::Noise => (blip.from * MUFFLE_HZ).max(150.0),
        _ => blip.from,
    };
    Blip {
        wave,
        from,
        to: if blip.wave == Wave::Noise {
            from
        } else {
            blip.to
        },
        gain: blip.gain * MUFFLE_LEVEL,
        ..blip
    }
}

/// Returns the blips of `call` and how many of them sound, each call a little different in pitch
/// and always in the rhythm of its voice and cry.
pub fn call(call: &Call, rand: &mut Rand) -> ([Blip; CALL_BLIPS], usize) {
    let beat = rhythm(&call.voice, call.cry);
    let hz = call.voice.hz * rise(call.cry) * rand.range(0.95, 1.05);
    let mut blips = [Blip::BASE; CALL_BLIPS];
    let grit = grit(call.voice.timbre, hz, beat.note);
    let notes = usize::from(beat.notes).min(CALL_BLIPS - usize::from(grit.is_some()));
    for (n, blip) in blips.iter_mut().enumerate().take(notes) {
        let lift = 1.0 + 0.06 * (n % 2) as f32;
        *blip = note(
            call.voice.timbre,
            hz * lift,
            beat.note,
            beat.period() * n as f32,
        );
    }
    let mut count = notes;
    if let Some(grit) = grit {
        blips[count] = grit;
        count += 1;
    }
    if call.voice.muffled {
        blips.iter_mut().take(count).for_each(|b| *b = muffle(*b));
    }
    (blips, count)
}
