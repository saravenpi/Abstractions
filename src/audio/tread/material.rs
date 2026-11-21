//! What each kind of ground answers a foot with. A footfall is an impact and what the ground
//! does after it: a thud of the ground giving way, a click of the sole meeting it, grains that
//! move for a moment (gravel rolls, snow creaks, sand and grass hiss, water drips), and on a
//! hollow or a hard floor a ring. Levels are peaks for the hardest step, against a full scale
//! of 1; frequencies are in hertz and times in seconds.

use crate::audio::cue::StepSurface;

/// The answer of one kind of ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    /// The thud: its tone and its level.
    pub thud: (f32, f32),
    /// The click of the contact: centre, resonance, seconds it takes to die, and level.
    pub click: (f32, f32, f32, f32),
    /// The grains: centre, resonance, grains per second right after the impact, seconds their
    /// motion takes to die, and level.
    pub grain: (f32, f32, f32, f32, f32),
    /// The ring of a hollow or hard floor under the metal sole: tone, resonance and level.
    pub ring: (f32, f32, f32),
}

/// Ground with no ring: loose or soft ground swallows the sole.
const DEAD: (f32, f32, f32) = (500.0, 4.0, 0.0);

/// Ground with no grains: a solid floor.
const SOLID: (f32, f32, f32, f32, f32) = (2000.0, 1.0, 0.0, 0.05, 0.0);

/// Returns the answer of `surface`.
pub const fn of(surface: StepSurface) -> Material {
    let (thud, click, grain, ring) = match surface {
        StepSurface::Sand => (
            (70.0, 0.05),
            (900.0, 0.7, 0.012, 0.02),
            (1600.0, 1.0, 9000.0, 0.09, 0.03),
            DEAD,
        ),
        StepSurface::Grass => (
            (80.0, 0.06),
            (700.0, 0.7, 0.010, 0.015),
            (4200.0, 0.8, 6000.0, 0.07, 0.022),
            DEAD,
        ),
        StepSurface::Rock => (
            (110.0, 0.06),
            (2600.0, 1.5, 0.004, 0.07),
            (4500.0, 2.0, 60.0, 0.05, 0.02),
            (640.0, 6.0, 0.012),
        ),
        StepSurface::Wood => ((165.0, 0.1), (1500.0, 1.2, 0.005, 0.045), SOLID, (410.0, 8.0, 0.02)),
        StepSurface::Snow => (
            (65.0, 0.035),
            (500.0, 0.6, 0.02, 0.01),
            (1900.0, 3.0, 900.0, 0.22, 0.06),
            DEAD,
        ),
        StepSurface::Gravel => (
            (90.0, 0.05),
            (2200.0, 1.0, 0.006, 0.03),
            (3200.0, 2.5, 500.0, 0.16, 0.07),
            DEAD,
        ),
        StepSurface::Water => (
            (90.0, 0.03),
            (800.0, 0.6, 0.03, 0.04),
            (1300.0, 4.0, 300.0, 0.3, 0.05),
            DEAD,
        ),
        StepSurface::Metal => ((120.0, 0.07), (3000.0, 1.5, 0.003, 0.06), SOLID, (520.0, 30.0, 0.035)),
    };
    Material {
        thud,
        click,
        grain,
        ring,
    }
}
