//! The recipe of every cue: a few blips each.

use super::chirps::chirp;
use super::steps::step;
use super::voice::{Blip, Wave};
use crate::audio::cue::AudioCue;
use crate::audio::dsp::rand::Rand;

/// Builds a pitched blip gliding from `from` to `to` hertz.
pub(super) const fn tone(from: f32, to: f32, seconds: f32, delay: f32, gain: f32) -> Blip {
    Blip {
        from,
        to,
        seconds,
        delay,
        gain,
        ..Blip::BASE
    }
}

/// Builds a noise burst centred on `hz` with a fade-in of `attack` seconds.
pub(super) const fn noise(hz: f32, seconds: f32, attack: f32, delay: f32, gain: f32) -> Blip {
    Blip {
        wave: Wave::Noise,
        from: hz,
        to: hz,
        seconds,
        attack,
        delay,
        gain,
        swell: false,
    }
}

/// Makes a blip pure, slow to start and growing to its end.
const fn swelling(blip: Blip) -> Blip {
    Blip {
        wave: Wave::Sine,
        attack: 0.05,
        swell: true,
        ..blip
    }
}

/// Makes a blip a pure sine with a slow start, for booms and thuds.
pub(super) const fn deep(blip: Blip) -> Blip {
    Blip {
        wave: Wave::Sine,
        attack: 0.012,
        ..blip
    }
}

/// Selection moved: one tiny high tick.
const UI_MOVE: [Blip; 1] = [tone(1320.0, 1245.0, 0.04, 0.0, 0.09)];

/// Accepted: E5 then B5, a fifth up.
const UI_CONFIRM: [Blip; 2] = [
    tone(659.3, 659.3, 0.13, 0.0, 0.13),
    tone(987.8, 987.8, 0.26, 0.085, 0.13),
];

/// Refused or closed: one low falling note.
const UI_CANCEL: [Blip; 1] = [tone(277.2, 196.0, 0.2, 0.0, 0.15)];

/// Landing: a low thud with some grit.
const LAND: [Blip; 2] = [
    deep(tone(88.0, 40.0, 0.38, 0.0, 0.32)),
    noise(190.0, 0.22, 0.006, 0.0, 0.16),
];

/// Warp charging: three tones rising for three seconds.
const WARP_SPOOL: [Blip; 3] = [
    swelling(tone(70.0, 520.0, 3.2, 0.0, 0.22)),
    swelling(tone(105.4, 783.0, 3.2, 0.0, 0.1)),
    swelling(tone(140.0, 1040.0, 3.2, 0.0, 0.06)),
];

/// Warp firing: a soft boom and a falling tone.
const WARP_ENGAGE: [Blip; 3] = [
    deep(tone(72.0, 30.0, 1.7, 0.0, 0.32)),
    noise(130.0, 1.2, 0.02, 0.0, 0.18),
    deep(tone(880.0, 220.0, 0.6, 0.0, 0.05)),
];

/// Leaving warp: a smaller boom and a tone falling back to rest.
const WARP_EXIT: [Blip; 3] = [
    deep(tone(58.0, 34.0, 1.2, 0.0, 0.26)),
    deep(tone(620.0, 92.0, 0.9, 0.0, 0.07)),
    noise(210.0, 0.5, 0.02, 0.0, 0.12),
];

/// Far thunder: a long low roll that swells in, with a rumble of noise trailing it.
const THUNDER: [Blip; 3] = [
    deep(tone(62.0, 34.0, 3.2, 0.0, 0.3)),
    noise(110.0, 2.8, 0.35, 0.05, 0.3),
    noise(70.0, 3.6, 0.6, 0.4, 0.25),
];

/// Returns the blips of a cue with the pitch factor and the level to play them at.
///
/// Footsteps and chirps vary a little each time so that they never sound stamped. The steps of
/// a machine vary less than those of a walker would.
pub fn recipe(cue: AudioCue, rand: &mut Rand) -> (&'static [Blip], f32, f32) {
    match cue {
        AudioCue::UiMove => (&UI_MOVE, 1.0, 1.0),
        AudioCue::UiConfirm => (&UI_CONFIRM, 1.0, 1.0),
        AudioCue::UiCancel => (&UI_CANCEL, 1.0, 1.0),
        AudioCue::Footstep(surface) => {
            (step(surface), rand.range(0.94, 1.06), rand.range(0.75, 1.0))
        }
        AudioCue::Land(strength) => (&LAND, 1.0, 0.25 + 0.75 * strength.clamp(0.0, 1.0)),
        AudioCue::WarpSpool => (&WARP_SPOOL, 1.0, 1.0),
        AudioCue::WarpEngage => (&WARP_ENGAGE, 1.0, 1.0),
        AudioCue::WarpExit => (&WARP_EXIT, 1.0, 1.0),
        AudioCue::Chirp(kind) => (chirp(kind), rand.range(0.95, 1.06), 1.0),
        AudioCue::Thunder(level) => (&THUNDER, rand.range(0.85, 1.1), level.clamp(0.0, 1.0)),
    }
}
