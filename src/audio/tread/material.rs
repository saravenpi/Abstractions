//! What each kind of ground answers a foot with. A footfall is an impact and what the ground
//! does after it: a thud of the ground giving way under the mass that strikes it, a click of
//! the sole meeting it, grains that break or move for a moment (blades swish, soil crumbles,
//! gravel and snow crunch, sand hisses, water drops), a ring on a hollow or a hard floor, and in
//! mud and water a sweep of wet noise as the foot goes in and as it pulls out.
//!
//! Levels are peaks for a walk at home, against a full scale of 1, frequencies are in hertz and
//! times in seconds.

use super::tune::{HISS_Q, KNEE, WET_Q};
use crate::audio::cue::StepSurface;

/// The answer of one kind of ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    /// The thud: its tone, its resonance and its level.
    pub thud: (f32, f32, f32),
    /// The click of the contact: centre, resonance, seconds it takes to die, and level.
    pub click: (f32, f32, f32, f32),
    /// The grains: lowest and highest centre, resonance, grains per second right after the
    /// strike, seconds their motion takes to die, and the peak of one grain.
    pub grain: (f32, f32, f32, f32, f32, f32),
    /// The ring of the floor: tone, resonance, level of a strike, and the share of each grain
    /// that sets it ringing too, which makes snow squeak.
    pub ring: (f32, f32, f32, f32),
    /// The wet sweep: the centre it starts from and the one it ends on as the foot goes in, the
    /// other way as it pulls out, its seconds, its level going in and pulling out, and the
    /// resonance of its band.
    pub wet: (f32, f32, f32, f32, f32, f32),
}

/// Ground with no ring: loose or soft ground swallows the sole.
const DEAD: (f32, f32, f32, f32) = (500.0, 4.0, 0.0, 0.0);

/// Ground with no grains: a solid floor.
const SOLID: (f32, f32, f32, f32, f32, f32) = (1000.0, 1000.0, 1.0, 0.0, 0.05, 0.0);

/// Dry ground: nothing wet.
const DRY: (f32, f32, f32, f32, f32, f32) = (500.0, 500.0, 0.1, 0.0, 0.0, WET_Q);

/// The answer of every ground, in the order of [`StepSurface::ALL`].
const GROUNDS: [Material; 10] = [
    Material {
        thud: (75.0, 1.5, 0.137),
        click: (900.0, 0.8, 0.008, 0.0328),
        grain: (2500.0, 6000.0, 1.2, 3500.0, 0.06, 0.0546),
        ring: DEAD,
        wet: DRY,
    },
    Material {
        thud: (68.0, 1.4, 0.15),
        click: (600.0, 0.7, 0.01, 0.0323),
        grain: (500.0, 1800.0, 1.5, 900.0, 0.05, 0.0537),
        ring: DEAD,
        wet: DRY,
    },
    Material {
        thud: (95.0, 1.5, 0.0497),
        click: (2000.0, 1.0, 0.004, 0.0284),
        grain: (1800.0, 5500.0, 4.0, 700.0, 0.12, 0.0994),
        ring: DEAD,
        wet: DRY,
    },
    Material {
        thud: (130.0, 2.5, 0.0576),
        click: (3200.0, 1.2, 0.003, 0.072),
        grain: (3000.0, 7000.0, 3.0, 80.0, 0.04, 0.0432),
        ring: (700.0, 5.0, 0.00864, 0.0),
        wet: DRY,
    },
    Material {
        thud: (110.0, 0.7, 0.03),
        click: (2500.0, 0.6, 0.006, 0.02),
        grain: (2000.0, 7000.0, 0.7, 5000.0, 0.06, 0.035),
        ring: DEAD,
        wet: DRY,
    },
    Material {
        thud: (60.0, 1.3, 0.0465),
        click: (500.0, 0.6, 0.02, 0.0124),
        grain: (900.0, 3200.0, 2.5, 800.0, 0.2, 0.0775),
        ring: (1300.0, 22.0, 0.0, 0.4),
        wet: DRY,
    },
    Material {
        thud: (50.0, 1.0, 0.0504),
        click: (400.0, 0.7, 0.02, 0.0072),
        grain: (300.0, 900.0, 8.0, 120.0, 0.25, 0.0216),
        ring: DEAD,
        wet: (420.0, 150.0, 0.14, 0.0288, 0.036, WET_Q),
    },
    Material {
        thud: (70.0, 0.7, 0.05),
        click: (2200.0, 0.5, 0.04, 0.08),
        grain: (1500.0, 5000.0, 3.0, 250.0, 0.3, 0.06),
        ring: DEAD,
        wet: (2500.0, 700.0, 0.18, 0.05, 0.03, HISS_Q),
    },
    Material {
        thud: (150.0, 5.0, 0.0755),
        click: (3500.0, 1.5, 0.003, 0.0604),
        grain: SOLID,
        ring: (640.0, 28.0, 0.0453, 0.0),
        wet: DRY,
    },
    Material {
        thud: (105.0, 9.0, 0.09),
        click: (2600.0, 1.2, 0.004, 0.054),
        grain: SOLID,
        ring: (430.0, 18.0, 0.045, 0.0),
        wet: DRY,
    },
];

/// Returns the answer of `surface`.
pub const fn of(surface: StepSurface) -> Material {
    GROUNDS[surface as usize]
}

/// Returns the answer of `surface` under a foot in water `depth` metres deep: only water wades.
pub fn ground(surface: StepSurface, depth: f32) -> Material {
    match surface {
        StepSurface::Water => wading(of(surface), depth),
        _ => of(surface),
    }
}

/// Returns `stuff` for a leg `depth` metres into the water. Toward the knee the splash, the thud
/// and the drops fade, since the foot no longer breaks the surface, and the slosh of the leg
/// lengthens into a swish that overlaps the next step and falls an octave as more water moves.
fn wading(stuff: Material, depth: f32) -> Material {
    let deep = (depth / KNEE).clamp(0.0, 1.0);
    let (thud, click, grain, wet) = (stuff.thud, stuff.click, stuff.grain, stuff.wet);
    let low = 1.0 / (1.0 + deep);
    Material {
        thud: (thud.0, thud.1, thud.2 * (1.0 - 0.7 * deep)),
        click: (click.0, click.1, click.2, click.3 * (1.0 - 0.8 * deep)),
        grain: (
            grain.0,
            grain.1,
            grain.2,
            grain.3 * (1.0 - 0.8 * deep),
            grain.4,
            grain.5,
        ),
        wet: (
            wet.0 * low,
            wet.1 * low,
            wet.2 * (1.0 + 3.0 * deep),
            wet.3,
            wet.4,
            wet.5,
        ),
        ..stuff
    }
}
