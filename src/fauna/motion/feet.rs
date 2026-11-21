//! The foot planner: planted feet never move, swinging feet arc to a landing predicted ahead.
//!
//! A foot lifts once per stride, when its own phase passes the duty factor and it is further than a
//! step from where it should land. A body standing still or turning on the spot steps its feet
//! back under it a few at a time. A foot stretched past its reach steps at once. A swinging foot
//! aims at its rest point carried ahead by the velocity over what is left of the swing and half the
//! stance, so the stance centres under the hip, and it lands exactly on the ground.

use std::f32::consts::PI;

use bevy::math::{Vec2, Vec3};

use super::gait::ramp;

/// Metres a planted foot must move when seated again for the ground under it to count as drawn anew,
/// over the rounding of reading the same ground through another chart slant.
const SEATED: f32 = 1e-3;

/// One foot of a leg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Foot {
    /// Where the foot is.
    pub at: Vec3,
    /// Where the swing started.
    from: Vec3,
    /// Where the swing lands.
    pub to: Vec3,
    /// Progress of the swing, 0 to 1.
    t: f32,
    /// Length of the swing in seconds.
    span: f32,
    /// Whether the foot carries the body: a planted foot does not move.
    pub planted: bool,
    /// Whether the foot may lift when its phase window opens, set during its stance.
    armed: bool,
}

/// What a body asks of all its feet this step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stride {
    /// Horizontal velocity in metres per second.
    pub vel: Vec3,
    /// Length of a swing in seconds.
    pub swing: f32,
    /// Length of a stance in seconds, capped for a slow body.
    pub stance: f32,
    /// Height of the swing arc in metres.
    pub lift: f32,
    /// Distance in metres from its landing a foot must be to step.
    pub step: f32,
    /// Distance in metres from its rest point past which a foot steps at once.
    pub reach: f32,
    /// Duty factor of the gait.
    pub duty: f32,
}

/// What one foot gets this step: its rest point under the body and its own phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cue {
    /// The rest point of the foot under the body now, on the ground plane of the body.
    pub rest: Vec3,
    /// Own phase of the foot, 0 to 1: planted below the duty factor.
    pub phase: f32,
    /// Whether the foot may step out of phase: the body is slow and few of its feet are up.
    pub free: bool,
}

/// Returns the horizontal distance between `a` and `b`.
fn flat(a: Vec3, b: Vec3) -> f32 {
    (a - b).with_y(0.0).length()
}

/// Returns the point at `x, z` on the ground `ground`.
pub fn on(ground: &dyn Fn(Vec2) -> f32, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, ground(Vec2::new(x, z)), z)
}

impl Foot {
    /// Returns a foot planted at `at`.
    pub fn new(at: Vec3) -> Self {
        Self {
            at,
            from: at,
            to: at,
            t: 1.0,
            span: 0.3,
            planted: true,
            armed: false,
        }
    }

    /// Returns whether the foot, planted, should lift now toward `target`.
    fn lifts(&mut self, cue: &Cue, stride: &Stride, target: Vec3) -> bool {
        if cue.phase < stride.duty {
            self.armed = true;
        }
        let far = flat(self.at, target) > stride.step;
        let window = self.armed && cue.phase >= stride.duty;
        if window && !far {
            self.armed = false;
        }
        (window && far) || (cue.free && far) || flat(self.at, cue.rest) > stride.reach
    }

    /// Moves the foot by `dt` seconds of `stride` on `ground` and returns whether it lifted.
    pub fn step(
        &mut self,
        cue: &Cue,
        stride: &Stride,
        ground: &dyn Fn(Vec2) -> f32,
        dt: f32,
    ) -> bool {
        let left = if self.planted {
            stride.swing
        } else {
            (1.0 - self.t) * self.span
        };
        let ahead = cue.rest + stride.vel * (left + 0.5 * stride.stance);
        let target = on(ground, ahead.x, ahead.z);
        if !self.planted {
            self.swing(target, stride.lift, ground, dt);
            return false;
        }
        if !self.lifts(cue, stride, target) {
            return false;
        }
        (self.planted, self.armed, self.from, self.to) = (false, false, self.at, target);
        (self.t, self.span) = (0.0, stride.swing);
        true
    }

    /// Carries a swinging foot toward `target` by `dt` seconds on an arc `lift` high, landing it on `ground`.
    fn swing(&mut self, target: Vec3, lift: f32, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
        self.to = target;
        self.t = (self.t + dt / self.span.max(1e-3)).min(1.0);
        if self.t >= 1.0 {
            (self.at, self.planted) = (target, true);
            return;
        }
        let ease = ramp(0.0, 1.0, self.t);
        let path = self.from.lerp(target, ease);
        let floor = ground(Vec2::new(path.x, path.z));
        self.at = path.with_y(path.y.max(floor) + lift * (PI * self.t).sin());
    }

    /// Sets a planted foot back on `ground` where it stands, and returns true when that moved it by
    /// more than [`SEATED`] metres: the drawn ground under it changed level as the chunks streamed. A
    /// swinging foot lands on the ground of its own.
    pub fn seat(&mut self, ground: &dyn Fn(Vec2) -> f32) -> bool {
        if !self.planted {
            return false;
        }
        let was = self.at.y;
        self.at = on(ground, self.at.x, self.at.z);
        self.to = self.at;
        (self.at.y - was).abs() > SEATED
    }

    /// Puts the foot where the phase alone says, on `ground` under where it is, moving along `dir`
    /// over a stride `length` long: the cheap foot of a distant body.
    pub fn sketch(
        &mut self,
        cue: &Cue,
        stride: &Stride,
        (ground, dir): (&dyn Fn(Vec2) -> f32, Vec3),
        length: f32,
    ) {
        let travel = stride.duty * length;
        let (ahead, up, planted) = match cue.phase < stride.duty {
            true => (travel * (0.5 - cue.phase / stride.duty), 0.0, true),
            false => {
                let s = (cue.phase - stride.duty) / (1.0 - stride.duty);
                (
                    -0.5 * travel * (PI * s).cos(),
                    stride.lift * (PI * s).sin(),
                    false,
                )
            }
        };
        let at = on(
            ground,
            cue.rest.x + dir.x * ahead,
            cue.rest.z + dir.z * ahead,
        );
        (self.at, self.to, self.planted) = (at + Vec3::Y * up, at, planted);
    }
}
