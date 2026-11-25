//! The blips of a blow or shot: a thud, a ring or a click, and a wet noise when it cut; one tick
//! for a glance; one band of noise for a swing; a deep boom, muzzle crack and barrel ring for a
//! shot.

use super::sounds::{deep, noise, tone};
use super::voice::Blip;
use crate::audio::dsp::rand::Rand;
use crate::audio::hit::{HitSound, Kind};

/// Most blips of one blow.
pub(super) const HIT_BLIPS: usize = 3;

/// Share of the ring that is the thud under it: a quarter.
const THUD: f32 = 0.25;

/// Brightness from which a blow rings instead of clicking.
const RINGS: f32 = 0.35;

/// Returns the blips of `sound` and how many of them there are, at most [`HIT_BLIPS`]. A little
/// variation in pitch keeps two equal blows from sounding stamped. Nothing is allocated: this runs
/// on the audio thread.
pub(super) fn hit(sound: &HitSound, rand: &mut Rand) -> ([Blip; HIT_BLIPS], usize) {
    let g = 0.08 + 0.4 * sound.level;
    let hz = sound.hz * rand.range(0.97, 1.03);
    let mut out = [Blip::BASE; HIT_BLIPS];
    match sound.kind {
        Kind::Whoosh => out[0] = noise(hz, 0.3, 0.1, 0.0, 0.5 * g),
        Kind::Tick => out[0] = tone((hz * 2.0).clamp(1500.0, 5000.0), 1200.0, 0.03, 0.0, g),
        Kind::Clean => return clean(sound, hz, g),
        Kind::Shot => return shot(sound, hz, g),
    }
    (out, 1)
}

/// Returns the blips of a blow that landed: the thud, then a ring on something hard or a click on
/// something soft, then a wet noise when it cut.
fn clean(sound: &HitSound, hz: f32, g: f32) -> ([Blip; HIT_BLIPS], usize) {
    let thud = (hz * THUD).clamp(70.0, 220.0);
    let body = deep(tone(
        thud,
        thud * 0.55,
        0.1 + 0.12 * (1.0 - sound.hard),
        0.0,
        0.9 * g,
    ));
    let top = if sound.hard >= RINGS {
        tone(
            hz,
            hz * 0.99,
            0.06 + 0.35 * sound.hard,
            0.0,
            0.5 * g * sound.hard,
        )
    } else {
        noise((hz * 3.0).clamp(500.0, 3000.0), 0.04, 0.003, 0.0, 0.3 * g)
    };
    let wet = noise(900.0, 0.09, 0.012, 0.01, 0.25 * g);
    ([body, top, wet], if sound.wet { 3 } else { 2 })
}

/// Returns the blips of a gunshot: a deep boom, a sharp muzzle crack and a metallic barrel ring.
fn shot(sound: &HitSound, hz: f32, g: f32) -> ([Blip; HIT_BLIPS], usize) {
    let boom = deep(tone(hz.clamp(70.0, 180.0), 32.0, 0.16, 0.0, 1.0 * g));
    let crack = noise((hz * 12.0).clamp(900.0, 2400.0), 0.14, 0.002, 0.0, 0.85 * g);
    let ring = tone(
        (hz * 4.5).clamp(400.0, 1400.0),
        140.0,
        0.08,
        0.005,
        0.4 * g * sound.hard.max(0.4),
    );
    ([boom, crack, ring], 3)
}
