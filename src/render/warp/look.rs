//! What the warp sequence looks and feels like: every value the sky, the view and the engines
//! take from it, each a pure function of the state.

use bevy::prelude::*;

use super::lens::Lens;
use super::{ENGAGED, Phase, Sequence};
use crate::render::celestial::smoothstep;

/// Share of the speed of light the sky is seen through at the top warp speed. The drive is far
/// faster than light; this is the speed the eye is shown, kept under 1 so the sky stays a sky.
pub const BETA_CAP: f32 = 0.85;

/// Share of that speed an engaged drive shows at rest: the charge gathers the sky, the velocity
/// adds the rest.
const GATHER: f32 = 0.7;

/// Seconds the view is exposed for: a star streaks by the angle it crosses in that time.
const EXPOSURE: f32 = 1.0 / 24.0;

/// Radians per second the sky at right angles to the heading flows by at the top warp speed.
const FLOW: f32 = 6.0;

/// Brightness of the light ahead at the top warp speed.
const CORE: f32 = 0.5;

/// Brightness of the flash of a drop-out, low and wide enough to be a breath of light.
const FLASH: f32 = 0.3;

/// Share of the drop-out over which its flash rises.
const FLASH_RISE: f32 = 0.15;

/// Velocities between which single stars pass too fast to follow and fade. The first is just
/// above the fastest cruise between the stars, the second about a hundred light-years a second.
const BLUR: (f32, f32) = (0.81, 0.9);

/// Share of their light single stars keep once they pass too fast to follow.
const BLURRED: f32 = 0.25;

/// Shake of the view, against the full shake of rough air, of a drive about to engage and of
/// a drive in cruise.
const TREMOR: (f32, f32) = (0.6, 0.15);

impl Sequence {
    /// Returns the speed the sky is seen through, as a share of the speed of light.
    pub fn beta(&self) -> f32 {
        let gathered = GATHER + (1.0 - GATHER) * self.velocity;
        BETA_CAP * smoothstep(0.0, 1.0, self.charge) * gathered
    }

    /// Returns the length in radians of the streak of a star at right angles to the heading:
    /// the angle the sky flows by during one exposure. At rest it is 0 and the streak is the star.
    pub fn streak(&self) -> f32 {
        FLOW * EXPOSURE * self.velocity
    }

    /// Returns the brightness of the light ahead.
    pub fn core(&self) -> f32 {
        CORE * self.velocity
    }

    /// Returns the brightness of the flash of the drop-out: nothing outside it, a soft rise as
    /// it starts, gone when the velocity is.
    pub fn flash(&self) -> f32 {
        if self.phase != Phase::DropOut {
            return 0.0;
        }
        let moving = (self.velocity / ENGAGED).min(1.0);
        FLASH * smoothstep(0.0, FLASH_RISE, self.progress) * moving
    }

    /// Returns the share of their light single stars keep: all of it up to the fastest cruise.
    pub fn stars(&self) -> f32 {
        1.0 - (1.0 - BLURRED) * smoothstep(BLUR.0, BLUR.1, self.velocity)
    }

    /// Returns how hard the drive shakes the view, from 0 to 1: it rises with the charge and
    /// settles low once the ship is under way.
    pub fn tremor(&self) -> f32 {
        let under_way = (self.velocity / ENGAGED).min(1.0);
        self.charge * (TREMOR.0 + (TREMOR.1 - TREMOR.0) * under_way)
    }

    /// Returns how far the engines are lit by the drive, from 0 to 1.
    pub fn thrust(&self) -> f32 {
        smoothstep(0.0, 1.0, self.charge)
    }

    /// Returns what the sky draws this state with when the ship flies along the unit vector `heading`.
    pub fn lens(&self, heading: Vec3) -> Lens {
        Lens {
            heading,
            level: self.charge.max(self.velocity),
            beta: self.beta(),
            streak: self.streak(),
            core: self.core(),
            flash: self.flash(),
            stars: self.stars(),
        }
    }
}
