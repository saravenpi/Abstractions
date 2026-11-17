//! The footsteps of the robot the player controls: light, mechanical and quiet.
//!
//! Every step is a soft servo tick, then a damped metallic tap of two partials that are not in
//! tune, like a struck bar, then the answer of the ground. The ground colours the step: sand
//! and grass swallow the upper partial, rock and wood ring it, snow crunches.

use super::sounds::{deep, noise, tone};
use super::voice::{Blip, Wave};
use crate::audio::cue::StepSurface;

/// Seconds between the servo tick and the foot touching down.
const REACH: f32 = 0.022;

/// Frequency of the upper partial of the foot over the lower one: the second mode of a bar.
const BAR: f32 = 2.76;

/// The servo of the leg: a faint whirr that rises as the foot comes down.
const SERVO: Blip = tone(1700.0, 2300.0, REACH, 0.0, 0.028);

/// Builds one partial of the tap of the metal foot: a pure tone that sags a little as it dies.
const fn metal(hz: f32, seconds: f32, gain: f32) -> Blip {
    Blip {
        wave: Wave::Sine,
        attack: 0.002,
        ..tone(hz, hz * 0.985, seconds, REACH, gain)
    }
}

/// Builds the answer of the ground: a noise burst that starts as the foot touches down.
const fn ground(hz: f32, seconds: f32, attack: f32, gain: f32) -> Blip {
    noise(hz, seconds, attack, REACH, gain)
}

/// Sand: the tap is dull and short, over a low scuff.
const STEP_SAND: [Blip; 4] = [
    SERVO,
    metal(430.0, 0.05, 0.06),
    metal(430.0 * BAR, 0.02, 0.008),
    ground(300.0, 0.07, 0.01, 0.05),
];

/// Grass: a dull tap under a soft brush.
const STEP_GRASS: [Blip; 4] = [
    SERVO,
    metal(480.0, 0.055, 0.06),
    metal(480.0 * BAR, 0.022, 0.01),
    ground(1500.0, 0.06, 0.008, 0.035),
];

/// Rock: the tap rings bright, with a hard click.
const STEP_ROCK: [Blip; 4] = [
    SERVO,
    metal(760.0, 0.08, 0.065),
    metal(760.0 * BAR, 0.06, 0.04),
    ground(3800.0, 0.012, 0.001, 0.045),
];

/// Wood: a bright tap over the hollow knock of the planks.
const STEP_WOOD: [Blip; 4] = [
    SERVO,
    metal(560.0, 0.07, 0.06),
    metal(560.0 * BAR, 0.045, 0.03),
    Blip {
        delay: REACH,
        ..deep(tone(170.0, 120.0, 0.06, 0.0, 0.08))
    },
];

/// Snow: a muted tap inside a two-grain crunch.
const STEP_SNOW: [Blip; 4] = [
    SERVO,
    metal(450.0, 0.045, 0.045),
    ground(2900.0, 0.09, 0.012, 0.06),
    noise(1300.0, 0.07, 0.015, REACH + 0.02, 0.045),
];

/// Returns the blips of a footstep.
pub(super) fn step(surface: StepSurface) -> &'static [Blip] {
    match surface {
        StepSurface::Sand => &STEP_SAND,
        StepSurface::Grass => &STEP_GRASS,
        StepSurface::Rock => &STEP_ROCK,
        StepSurface::Wood => &STEP_WOOD,
        StepSurface::Snow => &STEP_SNOW,
    }
}
