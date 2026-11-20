//! Steering: the drive an animal's motion gets from its place in the herd, the herd's own way and its
//! mood.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::motion::{Detail, Drive, Motion};
use super::react::{Act, Mood};
use super::tune::{FLEE_SPEED, PULL, SETTLE};
use crate::life::body::{Body, Locomotion};

/// Where an animal should be and how fast its herd goes, in the animal's frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    /// Its place now.
    pub place: Vec3,
    /// Velocity of its herd in metres per second.
    pub flow: Vec3,
    /// Alarm the herd itself is in, 0 to 1: hunters on a chase, prey hunted.
    pub excite: f32,
}

/// What the motion of an animal needs besides its aim: the planet's gravity, the animal's top speed
/// and the detail it is worth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pace {
    /// Gravity of the planet in metres per second squared.
    pub gravity: f32,
    /// Top speed of the species in metres per second.
    pub top: f32,
    /// Level of detail.
    pub detail: Detail,
}

/// Returns the height in metres a flyer of `body` cruises at above the ground.
pub fn cruise(body: &Body) -> f32 {
    (2.0 + 3.0 * body.length).clamp(2.0, 12.0)
}

/// Returns the velocity that keeps `at` on its herd place `aim`, no faster than `top`.
fn follow(at: Vec3, aim: &Aim, top: f32) -> Vec3 {
    let gap = (aim.place - at).with_y(0.0);
    if gap.length() < SETTLE && aim.flow.length() < 0.05 {
        return Vec3::ZERO;
    }
    (aim.flow.with_y(0.0) + gap * PULL).clamp_length_max(top)
}

/// Returns the point a grazing head looks at: the ground a little ahead of the body.
fn graze(motion: &Motion, body: &Body, ground: &dyn Fn(Vec2) -> f32) -> Vec3 {
    let at = motion.root.at;
    let ahead = at + motion.root.turn * Vec3::NEG_Z * (0.8 * body.length.max(0.1));
    ahead.with_y(ground(ahead.xz()))
}

/// Returns the drive of `motion` for `body` towards `aim` in `mood`, over `ground`.
pub fn drive(
    motion: &Motion,
    body: &Body,
    (aim, mood): (&Aim, &Mood),
    pace: Pace,
    ground: &dyn Fn(Vec2) -> f32,
) -> Drive {
    let at = motion.root.at;
    let towards = follow(at, aim, pace.top);
    let (velocity, look) = match mood.act() {
        Act::Graze if towards.length() < 0.3 => (towards, Some(graze(motion, body, ground))),
        Act::Graze => (towards, None),
        Act::Alert => (towards * 0.2, Some(mood.threat)),
        Act::Flee => {
            let away = (at - mood.threat).with_y(0.0).normalize_or(Vec3::X);
            (away * pace.top * FLEE_SPEED, Some(mood.threat))
        }
    };
    let climb = match body.locomotion {
        Locomotion::Fly | Locomotion::Swarm => ground(at.xz()) + cruise(body) - at.y,
        _ => 0.0,
    };
    Drive {
        velocity: velocity.with_y(climb),
        look,
        gravity: pace.gravity,
        excite: mood.alarm.max(aim.excite),
        detail: pace.detail,
    }
}
