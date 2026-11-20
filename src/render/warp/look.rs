//! What the warp sequence looks and feels like: every value the sky, the view and the engines
//! take from it, each a pure function of the state.

use bevy::prelude::*;

use super::lens::Lens;
use super::{ENGAGED, Phase, Sequence};
use crate::render::celestial::smoothstep;
use crate::universe::LY;

/// Share of the speed of light the sky is seen through at the top warp speed. The drive is far
/// faster than light; this is the speed the eye is shown, kept under 1 so the sky stays a sky.
pub const BETA_CAP: f32 = 0.85;

/// Share of that speed an engaged drive shows at rest: the charge gathers the sky, the velocity
/// adds the rest.
const GATHER: f32 = 0.7;

/// Seconds the view is exposed for: a star streaks by the angle it crosses in that time.
pub const EXPOSURE: f32 = 1.0 / 24.0;

/// Slowest warp speed and fastest jump speed in light-years per second: the ends of the
/// logarithmic scale the drive reports its speed on. The slowest is 2000 m/s, and the fastest is
/// 3,600 light-years per second, the cap of a jump to another galaxy. `ship::warp::fraction` is
/// the same scale in metres per second, and a ship test holds the two together.
pub const SPEEDS: (f32, f32) = ((2000.0 / LY) as f32, 3600.0);

/// Brightness of the flash of a drop-out, low and wide enough to be a breath of light.
const FLASH: f32 = 0.3;

/// Share of the drop-out over which its flash rises.
const FLASH_RISE: f32 = 0.15;

/// Velocities between which single stars pass too fast to follow and fade. The first is just
/// above the fastest cruise between the stars, the second about a hundred light-years a second.
const BLUR: (f32, f32) = (0.81, 0.93);

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

    /// Returns the speed the sky is drawn at, in game light-years per second: the speed the
    /// drive reports, read back from its logarithmic scale, and eased with the velocity while
    /// the ship gathers speed and while it drops out, so that it never steps.
    pub fn speed(&self) -> f32 {
        let gathering = (self.velocity / ENGAGED).clamp(0.0, 1.0);
        let pace = ((self.velocity - ENGAGED) / (1.0 - ENGAGED)).clamp(0.0, 1.0);
        gathering * SPEEDS.0 * (SPEEDS.1 / SPEEDS.0).powf(pace)
    }

    /// Returns the game light-years the ship flies during one exposure. A star streaks by the
    /// angle that flight moves it across the sky, which its own distance and bearing set; at
    /// rest it is 0 and the streak is the star.
    pub fn streak(&self) -> f32 {
        self.speed() * EXPOSURE
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
            core: 0.0,
            flash: self.flash(),
            stars: self.stars(),
            flow: self.flow(),
            dust: self.dust(),
            phase: 0.0,
        }
    }
}
