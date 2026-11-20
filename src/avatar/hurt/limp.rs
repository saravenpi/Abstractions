//! A hurt body moves differently: a limp in the walk, a crawl on two lost legs and a broken arm
//! that hangs.

use super::super::Mode;
use super::super::pose::{Core, Limb, Pose};
use super::tune::{CRAWL_LEAN, HANG, LIMP_DIP, LIMP_ROLL, LIMP_SPEED, STRIDE_LOSS};
use crate::condition::{Condition, Part};

/// Returns true when a body in `mode` is on its feet: standing, walking, running or touching down.
pub fn on_feet(mode: Mode) -> bool {
    matches!(mode, Mode::Idle | Mode::Walk | Mode::Run | Mode::Land)
}

/// Returns `pose` of a body walking at `speed` metres per second with the legs and arms of
/// `condition`: a hurt leg swings less and the body drops and rolls toward the sound one, two lost
/// legs lean the body forward, and a broken arm hangs.
pub fn limp(mut pose: Pose, condition: &Condition, speed: f32) -> Pose {
    let moving = (speed / LIMP_SPEED).clamp(0.0, 1.0);
    let weak = [Part::LegL, Part::LegR].map(|leg| 1.0 - condition.of(leg));
    for (side, weak) in weak.into_iter().enumerate() {
        let keep = 1.0 - weak * STRIDE_LOSS;
        for joint in [Limb::Hip, Limb::Knee] {
            pose.sides[side][joint as usize] *= keep;
        }
        if condition.broken(side) {
            for (joint, value) in [Limb::Shoulder, Limb::ArmOut, Limb::Elbow]
                .into_iter()
                .zip(HANG)
            {
                pose.sides[side][joint as usize] = value;
            }
        }
    }
    let weak = [Part::LegL, Part::LegR].map(|leg| 1.0 - condition.of(leg));
    let roll = pose.core(Core::Roll) + LIMP_ROLL * (weak[1] - weak[0]) * moving;
    let dip = pose.core(Core::Lift) - LIMP_DIP * (weak[0] + weak[1]) * moving;
    let lean = pose.core(Core::Pitch)
        + if condition.crawling() {
            CRAWL_LEAN
        } else {
            0.0
        };
    pose.with(Core::Roll, roll)
        .with(Core::Lift, dip)
        .with(Core::Pitch, lean)
        .bounded()
}
