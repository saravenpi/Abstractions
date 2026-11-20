//! Steering: the drive an animal's motion gets from its place in the herd, the herd's own way and its
//! mood.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::motion::{Detail, Drive, Motion};
use super::react::{Act, Mood};
use super::tune::{APPROACH_SPEED, FLEE_SPEED, PULL, SETTLE, STAND_OFF};
use crate::life::body::{Body, Locomotion, slope};

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

/// Returns the height in metres a flyer or a swarm of `body` cruises at above the ground: swarms
/// hover low over the plants, flyers circle well over the robot's head.
pub fn cruise(body: &Body) -> f32 {
    match body.locomotion {
        Locomotion::Swarm => (0.6 + 2.0 * body.length).clamp(0.5, 2.0),
        _ => (8.0 + 8.0 * body.length).clamp(8.0, 30.0),
    }
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

/// Returns the velocity and the look of `motion` for `body` doing what `mood` calls for, `towards`
/// being the velocity that keeps it on its herd place.
fn motive(
    (motion, body): (&Motion, &Body),
    (towards, mood): (Vec3, &Mood),
    top: f32,
    ground: &dyn Fn(Vec2) -> f32,
) -> (Vec3, Option<Vec3>) {
    let at = motion.root.at;
    let gap = (mood.threat - at).with_y(0.0);
    match mood.act() {
        Act::Graze if towards.length() < 0.3 => (towards, Some(graze(motion, body, ground))),
        Act::Graze => (towards, None),
        Act::Alert => (towards * 0.2, Some(mood.threat)),
        Act::Threaten => (Vec3::ZERO, Some(mood.threat)),
        Act::Approach if gap.length() < STAND_OFF + body.length => (Vec3::ZERO, Some(mood.threat)),
        Act::Approach => (
            gap.normalize_or_zero() * top * APPROACH_SPEED,
            Some(mood.threat),
        ),
        Act::Flee => (
            -gap.normalize_or(Vec3::NEG_X) * top * FLEE_SPEED,
            Some(mood.threat),
        ),
    }
}

/// Returns the slope of `ground` at `p` as its uphill gradient, rise over run, read over `span` metres.
fn gradient(ground: &dyn Fn(Vec2) -> f32, p: Vec2, span: f32) -> Vec2 {
    let dx = ground(p + Vec2::X * span) - ground(p - Vec2::X * span);
    let dz = ground(p + Vec2::Y * span) - ground(p - Vec2::Y * span);
    Vec2::new(dx, dz) / (2.0 * span)
}

/// Returns `velocity` of a body of `body` at `at` held to the ground it can walk. Where the ground one
/// stride ahead is steeper than [`slope`] allows, the part of the velocity up or down that slope is
/// taken out, so the body follows the foot or the edge of a cliff instead of passing it. A body that
/// already stands on ground too steep may still walk down it, never up.
fn hold(velocity: Vec3, at: Vec3, body: &Body, ground: &dyn Fn(Vec2) -> f32) -> Vec3 {
    let limit = slope(body).tan();
    let flat = velocity.xz();
    if !limit.is_finite() || flat.length() < 1e-4 {
        return velocity;
    }
    let reach = body.height.max(0.1);
    let here = gradient(ground, at.xz(), 0.5 * reach);
    let ahead = gradient(ground, at.xz() + flat.normalize() * reach, 0.5 * reach);
    let kept = match (here.length() > limit, ahead.length() > limit) {
        (true, _) => flat - here.normalize() * flat.dot(here.normalize()).max(0.0),
        (false, true) => flat - ahead.normalize() * flat.dot(ahead.normalize()),
        (false, false) => flat,
    };
    Vec3::new(kept.x, velocity.y, kept.y)
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
    let (velocity, look) = motive((motion, body), (towards, mood), pace.top, ground);
    let climb = match body.locomotion {
        Locomotion::Fly | Locomotion::Swarm => ground(at.xz()) + cruise(body) - at.y,
        _ => 0.0,
    };
    Drive {
        velocity: hold(velocity.with_y(climb), at, body, ground),
        look,
        gravity: pace.gravity,
        excite: mood.alarm.max(aim.excite),
        detail: pace.detail,
    }
}
