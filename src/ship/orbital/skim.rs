//! The orbital cruise as plain math: the drive carries the ship along a great circle of the
//! planet at a height it holds above the air, in axes centred on the planet that do not turn
//! with it.
//!
//! The speed at full lever is [`RATE`] radians of the planet per second. It rises from the
//! slowest warp speed by the factor `e` every [`RAMP`] seconds, as the warp drive does, and
//! toward a target it falls to the distance left over [`RAMP`], so the ship stops where the glide
//! begins and never past it.

use bevy::math::{DMat3, DQuat, DVec3};

use super::glide::remaining;
use super::tune::{CLIMB, LOCK_SLACK, RATE, TURN};
use crate::ship::route::shell::lock;
use crate::ship::warp::{FLOOR, RAMP};
use crate::ship::wells::PLANET_LOCK;
use crate::universe::PlanetSpec;

/// Where the cruise may fly over one planet: its sea level radius and the heights in metres
/// above it between which the cruise holds, from the stand-off of the arrival shell, just
/// outside the air, to the edge of the lock sphere, past which the warp drive is free.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    /// Sea level radius in metres.
    pub radius: f64,
    /// Lowest height held, in metres above sea level.
    pub floor: f64,
    /// Highest height held, in metres above sea level.
    pub ceiling: f64,
}

impl Band {
    /// Returns the band of `planet`.
    pub fn of(planet: &PlanetSpec) -> Band {
        let floor = lock(planet) - planet.radius;
        Band {
            radius: planet.radius,
            floor,
            ceiling: (planet.radius * (PLANET_LOCK - 1.0 - LOCK_SLACK)).max(floor),
        }
    }

    /// Returns the speed in metres per second of the full lever at `height` metres.
    pub fn top(&self, height: f64) -> f64 {
        RATE * (self.radius + height)
    }
}

/// The ship in cruise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Skim {
    /// Metres from the centre of the planet.
    pub pos: DVec3,
    /// Unit direction of flight, square to `pos`.
    pub heading: DVec3,
    /// Speed along the track in metres per second.
    pub speed: f64,
    /// Height held, in metres above sea level.
    pub height: f64,
}

/// What the pilot or the computer asks of one step of the cruise.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ask {
    /// Lever from 0 to 1: the share of the top speed.
    pub throttle: f64,
    /// Vertical stick from -1 (down) to 1 (up): the held height moves.
    pub climb: f64,
    /// Yaw stick from -1 (right) to 1 (left): the heading turns.
    pub yaw: f64,
    /// Unit direction of the target from the centre of the planet, when the cruise flies to one.
    pub goal: Option<DVec3>,
}

/// What one step of the cruise did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stride {
    /// The ship after the step.
    pub skim: Skim,
    /// Velocity of the step in metres per second.
    pub vel: DVec3,
    /// Whether the ship reached the point where the glide toward the target begins.
    pub done: bool,
    /// Whether the pilot asks to go down while the cruise is at its floor: the cruise ends there.
    pub floor: bool,
}

/// Returns the course of the cruise of `skim` for a step of `dt` seconds: along the great circle
/// to the goal, or its heading turned by the yaw stick, square to the vertical either way. The
/// ship turns onto it at its own rate; the step flies the heading the nose has.
pub fn course(skim: &Skim, ask: &Ask, dt: f64) -> DVec3 {
    let up = skim.pos.normalize_or(DVec3::Y);
    let wanted = match ask.goal {
        Some(goal) => (goal - up * goal.dot(up))
            .try_normalize()
            .unwrap_or(skim.heading),
        None => DQuat::from_axis_angle(up, ask.yaw * TURN * dt) * skim.heading,
    };
    (wanted - up * wanted.dot(up))
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector())
}

/// Returns the height held after the step. Toward a goal it goes down to the floor, otherwise
/// it follows the vertical stick; it never leaves the band.
fn held(height: f64, band: &Band, ask: &Ask, dt: f64) -> f64 {
    let climb = ask.goal.map_or(ask.climb.clamp(-1.0, 1.0), |_| -1.0);
    let rate = ((height - band.floor).max(0.0) * CLIMB.0).max(CLIMB.1);
    (height + climb * rate * dt).clamp(band.floor, band.ceiling)
}

/// Returns the speed of the step after `last`, at most `top`, and, with `left` metres to go,
/// slowing to the distance left over [`RAMP`] and never past it.
fn pace(last: f64, top: f64, left: Option<f64>, dt: f64) -> f64 {
    let rise = last.max(FLOOR) * (dt / RAMP).exp();
    let close = left.map_or(f64::INFINITY, |left| {
        (left / RAMP).max(FLOOR).min(left / dt.max(1e-9))
    });
    top.max(0.0).min(rise).min(close)
}

/// Returns the ship `distance` metres along the great circle from the unit direction `up` along
/// `heading`, at `radius` metres from the centre, and its heading there.
fn advance(up: DVec3, heading: DVec3, radius: f64, distance: f64) -> (DVec3, DVec3) {
    let axis = up.cross(heading).normalize_or(up.any_orthonormal_vector());
    let turn = DQuat::from_axis_angle(axis, distance / radius);
    ((turn * up).normalize() * radius, turn * heading)
}

/// Returns one step of `dt` seconds of the cruise of `skim` over `band`, along its heading, as
/// `ask` asks of its speed and its height.
pub fn step(skim: &Skim, band: &Band, ask: &Ask, dt: f64) -> Stride {
    let up = skim.pos.normalize_or(DVec3::Y);
    let heading =
        (skim.heading - up * skim.heading.dot(up)).normalize_or(up.any_orthonormal_vector());
    let height = held(skim.height, band, ask, dt);
    let left = ask
        .goal
        .map(|goal| remaining(up, goal, band.radius, height));
    let speed = pace(skim.speed, band.top(height) * ask.throttle, left, dt);
    let (pos, heading) = advance(up, heading, band.radius + height, speed * dt);
    Stride {
        skim: Skim {
            pos,
            heading,
            speed,
            height,
        },
        vel: (pos - skim.pos) / dt.max(1e-9),
        done: left.is_some_and(|left| left <= speed * dt + 1.0),
        floor: ask.goal.is_none() && ask.climb < 0.0 && skim.height <= band.floor + 1.0,
    }
}

/// Returns the orientation of a ship whose nose points along `ahead` with its roof toward `up`,
/// the nose turned down by `pitch` radians.
pub fn attitude(up: DVec3, ahead: DVec3, pitch: f64) -> DQuat {
    let flat = (ahead - up * ahead.dot(up)).normalize_or(up.any_orthonormal_vector());
    let level = DQuat::from_mat3(&DMat3::from_cols(flat.cross(up), up, -flat));
    level * DQuat::from_rotation_x(-pitch)
}
