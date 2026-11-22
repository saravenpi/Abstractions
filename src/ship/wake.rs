//! The trails a ship leaves in flight: a vortex trail from each wing tip while the wing makes
//! lift in air, a thin streak of light behind each navigation lamp, a contrail behind the engines
//! in cold air and the vapour cone that wraps the hull through Mach 1 in wet air.
//!
//! Everything is read from how the hull moves, frame after frame, and from the air at that
//! place: the speed, the load on the wing, the thickness and the wetness of the air. The ship
//! the player flies and the ship of another player are read the same way, from where they are
//! drawn, so both show the same trails for the same flight and the records carry nothing more.
//! The ribbons are the ones of the wingsuit: [`crate::gear::Trail`] and its strip.

pub(super) mod cone;
mod field;
mod lay;
mod sky;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_arc;
#[cfg(test)]
mod tests_sky;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::ShipSet;
use super::exhaust::Exhaust;
use crate::render::smoothstep as smooth;
use crate::speed::vortex;

/// Airspeeds between which the wing tips start to trail and trail fully, in metres per second:
/// under the first the ship hangs on its lift nozzles and the wing carries nothing.
pub const VORTEX_SPEED: (f32, f32) = (30.0, 70.0);

/// Glow of a wing tip in steady level flight, before a turn or a pull-up adds its load.
pub const STEADY: f32 = 0.5;

/// Loads on the wing, in g, between which a tip starts to trail and trails as in level flight:
/// a ship that coasts on a ballistic arc carries nothing on its wing and leaves no vortex.
pub const LIFTING: (f32, f32) = (0.3, 0.9);

/// Speeds between which a lamp starts to leave a streak and leaves all of it, in metres per second.
pub const LAMP_SPEED: (f32, f32) = (8.0, 30.0);

/// Seconds the streak of a lamp lasts in a vacuum and in air as thick as at home. In a vacuum
/// nothing scatters the light, so the streak is only what the lamp leaves in the eye, and it is
/// kept short; in air the wake of the tip glows in the light of the lamp for a little longer.
pub const LAMP_LIFE: (f32, f32) = (0.12, 0.6);

/// Share of the streak of a lamp that full daylight takes away: it shows best at night.
pub const LAMP_DAY: f32 = 0.7;

/// Seconds over which the load on the wing follows what the path of the hull says.
pub const EASE: f32 = 0.25;

/// The flight of a ship and the air around it, as its trails read them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Air {
    /// Speed through the air in metres per second.
    pub speed: f32,
    /// Load on the wing in g: 1 in level flight, more in a turn or a pull-up, 0 in free fall.
    pub load: f32,
    /// Thickness of the air against the sea level at home, from 0 in a vacuum to 1.
    pub thick: f32,
    /// Wetness of the air, from 0 (dry) to 1 (in cloud or rain).
    pub moist: f32,
    /// Daylight, from 0 at night to 1 at noon.
    pub day: f32,
    /// Speed against the speed of sound in that air.
    pub mach: f32,
    /// Temperature of the air in kelvin.
    pub cold: f32,
    /// Pressure of the air in pascals.
    pub pressure: f32,
    /// Share of the sky the clouds cover, which says how wet the air is aloft.
    pub cover: f32,
    /// Level of the main engines, from 0 at idle to 1 at full throttle and above.
    pub engine: f32,
}

/// Returns the load on the wing of a hull that moves at `vel` and gains `accel` metres per
/// second squared, where `up` is the unit vertical and `gravity` the pull of the planet: what
/// the hull feels across its path, in g. Level and steady it is 1, in free fall 0.
pub fn load(vel: Vec3, accel: Vec3, up: Vec3, gravity: f32) -> f32 {
    let felt = accel + up * gravity;
    let along = vel.normalize_or_zero();
    (felt - along * felt.dot(along)).length() / gravity.max(1.0e-3)
}

/// Returns how strongly a wing tip trails: nothing slow, nothing without air, nothing when the
/// wing carries no load, plain in level flight, brighter in a turn or a pull-up, and brighter
/// still where the air is wet enough for the vortex to condense.
pub fn vortex_heat(air: &Air) -> f32 {
    let pull = STEADY + (1.0 - STEADY) * smooth(1.1, 2.5, air.load);
    smooth(VORTEX_SPEED.0, VORTEX_SPEED.1, air.speed)
        * air.thick.clamp(0.0, 1.0)
        * smooth(LIFTING.0, LIFTING.1, air.load)
        * pull
        * vortex(air.moist)
}

/// Returns how strongly a navigation lamp leaves a streak: nothing at rest, all of it in fast
/// flight, less by day.
pub fn lamp_heat(air: &Air) -> f32 {
    smooth(LAMP_SPEED.0, LAMP_SPEED.1, air.speed) * (1.0 - LAMP_DAY * air.day.clamp(0.0, 1.0))
}

/// Returns the seconds the streak of a lamp lasts in air of thickness `thick`, from 0 to 1.
pub fn lamp_life(thick: f32) -> f32 {
    LAMP_LIFE.0 + (LAMP_LIFE.1 - LAMP_LIFE.0) * thick.clamp(0.0, 1.0)
}

/// How a hull moved on the frame before, kept on every ship that leaves trails.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Flown {
    /// The anchor frame and the frame position of the hull on the frame before.
    pub(super) was: Option<(crate::space::Frame, bevy::math::DVec3)>,
    /// Its velocity on the frame before, when it had one.
    vel: Option<Vec3>,
    /// The load on its wing, eased.
    pub load: f32,
    /// The air it flew in on this frame, which its vapour cone reads.
    pub air: Air,
}

impl Default for Flown {
    /// A hull that was nowhere yet, in level flight.
    fn default() -> Self {
        Self {
            was: None,
            vel: None,
            load: 1.0,
            air: Air::default(),
        }
    }
}

impl Flown {
    /// Takes the velocity `vel` of this frame, `dt` seconds after the last, under the vertical
    /// `up` and the pull `gravity`, and returns the speed and the eased load on the wing. A
    /// hull without a velocity, just placed or in another frame, starts over in level flight.
    pub fn advance(
        &mut self,
        vel: Option<Vec3>,
        (up, gravity): (Vec3, f32),
        dt: f32,
    ) -> (f32, f32) {
        let (Some(now), true) = (vel, dt > 0.0) else {
            *self = Self {
                was: self.was,
                ..Self::default()
            };
            return (0.0, self.load);
        };
        let accel = self.vel.map_or(Vec3::ZERO, |before| (now - before) / dt);
        let felt = load(now, accel, up, gravity);
        self.load += (felt - self.load) * (1.0 - (-dt / EASE).exp());
        self.vel = Some(now);
        (now.length(), self.load)
    }
}

/// Registers the trails: every ship gets its ribbons and its cone, and they are fed once the
/// ships are placed for the frame.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (lay::spawn, cone::spawn, lay::draw, cone::draw)
            .chain()
            .in_set(ShipSet::Report)
            .run_if(any_with_component::<Exhaust>),
    )
    .add_systems(
        PostUpdate,
        cone::follow
            .after(TransformSystems::Propagate)
            .run_if(any_with_component::<cone::Vapour>),
    );
}
