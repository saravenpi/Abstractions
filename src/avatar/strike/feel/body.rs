//! The body behind a strike: the weight on the back foot in the wind-up and onto the front foot
//! through the blow, the knees giving, the body stooping to a low target and the free arm swinging
//! against the tool. A heavy tool is swung from the legs, a light one from the arm.

use std::f32::consts::PI;

use super::super::super::gait::Foot;
use super::super::super::ground::{Stance, planted};
use super::super::super::pose::{Core, Limb, Pose};
use super::super::super::{AvatarPose, Mode};
use super::super::act::{Act, Strike, Timing};
use super::super::held::{Held, Style};
use super::super::layer::Layer;
use super::super::tune::{COAST, DRAW, GUARD_IN};
use super::bend::Bend;
use super::tune::{COUNTER, DIP, EASE, HEFT, LEAN, LUNGE_STEP, STAGGER, STEP, STOOP, SWAY};
use crate::craft::build::Hands;

/// The joints of a leg the strike sets.
const LEGS: [Limb; 4] = [Limb::Hip, Limb::LegOut, Limb::Knee, Limb::Ankle];

/// The joints of the trunk the stance sets.
const HIPS: [Core; 3] = [Core::Lift, Core::Sway, Core::Pitch];

/// The joints of the free arm.
const ARM: [Limb; 3] = [Limb::Shoulder, Limb::ArmOut, Limb::Elbow];

/// Returns how far `act` timed by `timing` is into its strike, 0 at rest to 1 in the blow, and its
/// coil, -1 cocked to 0 at rest to 1 followed through.
pub fn phase(act: Act, timing: &Timing) -> (f32, f32) {
    let s = |t: f32, length: f32| (t / length.max(f32::EPSILON)).clamp(0.0, 1.0);
    match act {
        Act::Ready | Act::Stow(_) => (0.0, 0.0),
        Act::Windup(t) => (s(t, timing.windup), -s(t, timing.windup)),
        Act::Swing(t) => (1.0, 2.0 * s(t, timing.swing) - 1.0),
        Act::Stun(_, at) => (1.0, (2.0 * at - 1.0).clamp(-1.0, 1.0)),
        Act::Recover(t) => (1.0 - s(t, timing.recover), 1.0 - s(t, timing.recover)),
        Act::Guard(t) => (0.5 * s(t, GUARD_IN), 0.0),
        Act::Draw(t) => (s(t, DRAW), -(0.5 * PI * s(t, DRAW)).sin()),
    }
}

/// Returns how much of a stance the legs of a body posed by `pose` take: all of it standing, none
/// walking fast, crouched or off its feet.
pub fn still(pose: &AvatarPose) -> f32 {
    let feet = matches!(pose.mode, Mode::Idle | Mode::Walk | Mode::Run | Mode::Land);
    let slow = 1.0 - ((pose.speed - 0.3) / 1.2).clamp(0.0, 1.0);
    match feet {
        true => slow * (1.0 - pose.crouch.clamp(0.0, 1.0)),
        false => 0.0,
    }
}

/// Returns the stance at `coil` of a strike bent by `bend`, with a tool of `heft` (0 light to 1 of
/// great inertia) that steps `step` metres into the blow: the front foot ahead and stepping in, the
/// weight back then forward, the knees giving through the blow and the body stooped to a low aim.
pub fn stance(coil: f32, bend: &Bend, (heft, step): (f32, f32)) -> Stance {
    let amp = HEFT.0 + HEFT.1 * heft;
    let (back, through) = ((-coil).max(0.0), coil.max(0.0));
    let foot = |ahead: f32| Foot {
        ahead,
        lift: 0.0,
        planted: true,
    };
    Stance {
        lift: -STOOP[0] * bend.low - DIP * (0.8 + amp * (0.5 * back + through)),
        lean: STOOP[1] * bend.low + amp * (LEAN.1 * through - LEAN.0 * back),
        sway: SWAY * amp * (back - through),
        feet: [
            foot(STAGGER + step * coil.abs()),
            foot(-STAGGER - 0.5 * step * back),
        ],
        roll: 0.0,
    }
}

/// Returns the shoulder, lift and elbow of the free arm at `coil`: reaching ahead while the tool is
/// cocked, tucked back while it follows through.
pub fn counter(coil: f32) -> [f32; 3] {
    let [ahead, rest, tuck] = COUNTER;
    let (to, k) = match coil < 0.0 {
        true => (ahead, -coil),
        false => (tuck, coil),
    };
    std::array::from_fn(|i| rest[i] + (to[i] - rest[i]) * k.min(1.0))
}

impl Layer {
    /// Returns this layer with the body of a strike `engage` into it at `coil`, the free arm swung
    /// against it when `free`, bent by `bend`, for a tool of `heft` stepping `step` into the blow.
    pub fn with_body(
        mut self,
        (engage, coil, free): (f32, f32, bool),
        bend: &Bend,
        (heft, step): (f32, f32),
    ) -> Self {
        self.engage = engage;
        self.coil = coil;
        self.legs = engage * bend.still;
        self.stance = stance(coil, bend, (heft, step));
        self.free = (free && self.arms[0].is_none()).then(|| counter(coil));
        self
    }

    /// Returns this layer with the body of `strike` with `held` timed by `timing` and bent by
    /// `bend`, eased from the layer `before` over `dt` seconds so the legs never jump.
    pub fn bodied(
        self,
        (strike, held): (&Strike, &Held),
        (bend, timing): (&Bend, &Timing),
        (before, dt): (&Layer, f32),
    ) -> Self {
        let (engage, coil) = phase(strike.act, timing);
        let rate = (EASE * dt).clamp(0.0, 1.0);
        let ease = |was: f32, now: f32| was + (now - was) * rate;
        let heft = held.swing.inertia / (held.swing.inertia + COAST);
        let step = match held.style == Style::Thrust || held.thrown() {
            true => STEP * LUNGE_STEP,
            false => STEP,
        };
        let free = held.build.hands == Hands::One && strike.hurt.usable[0];
        let eased = (ease(before.engage, engage), ease(before.coil, coil), free);
        self.with_body(eased, bend, (heft, step))
    }
}

/// Returns `pose` with the legs and the hips blended `w` of the way to where `stance` puts them.
fn legs(pose: Pose, stance: &Stance, w: f32) -> Pose {
    let to = planted(stance);
    let mut out = pose;
    for i in HIPS.map(|c| c as usize) {
        out.core[i] += (to.core[i] - out.core[i]) * w;
    }
    for side in 0..2 {
        for i in LEGS.map(|l| l as usize) {
            out.sides[side][i] += (to.sides[side][i] - out.sides[side][i]) * w;
        }
    }
    out
}

/// Returns `pose` with the body of `layer` laid over it at weight `w`: the stance of the legs and
/// the free arm.
pub fn stand(pose: Pose, layer: &Layer, w: f32) -> Pose {
    let mut out = match layer.legs * w > 0.0 {
        true => legs(pose, &layer.stance, layer.legs * w),
        false => pose,
    };
    if let Some(free) = layer.free {
        let k = (layer.engage * w).clamp(0.0, 1.0);
        for (joint, value) in ARM.into_iter().zip(free) {
            let base = out.sides[0][joint as usize];
            out.sides[0][joint as usize] = base + (value - base) * k;
        }
    }
    out
}
