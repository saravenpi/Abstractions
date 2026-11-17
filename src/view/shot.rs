//! The shots: the shape of the rig for each kind of target and each view, and the blends between them.
//!
//! A first person view is a third person view whose boom has no length, so a switch between
//! the two slides the camera along its own axis.

use bevy::prelude::*;

use super::io::{Kind, Target};

/// Height of the neck joint above the centre of the body capsule: the pivot the head turns on.
const NECK: f32 = 0.48;

/// From the neck to the eyes in the axes the camera looks along: right, up, back. The eyes sit
/// on the face of the head, ahead of the chest, so the view never looks down into the neck, and
/// they swing forward over the chest when the head looks down.
const FACE: Vec3 = Vec3::new(0.0, 0.22, -0.16);

/// Height above the centre of the body of the pivot the third person camera orbits: just above the shoulders.
const SHOULDERS: f32 = 0.8;

/// Pilot seat of the ship as multiples of its radius: under the canopy, ahead of the centre.
const SEAT: Vec3 = Vec3::new(0.0, 0.2, -0.29);

/// Boom of the chase camera as multiples of the radius of the ship: above and behind. From there
/// the sphere that holds the ship fits in a 75 degree field of view.
const CHASE: Vec3 = Vec3::new(0.0, 0.75, 2.9);

/// The shape of the rig at one instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Shot {
    /// From the centre of the target to the pivot, in the axes of the target.
    pub seat: Vec3,
    /// From the pivot to the camera, in the axes the camera looks along: right, up, back.
    pub boom: Vec3,
    /// Farthest the camera lags behind a moving target.
    pub lag: f32,
    /// Stiffness of the spring that carries that lag.
    pub stiff: f32,
    /// Rate at which the camera catches up with the rotation of the target.
    pub turn: f32,
    /// Share of the bank of the body the camera rolls with.
    pub bank: f32,
    /// Share of the angle to the flight path the camera turns by.
    pub lead: f32,
    /// Field of view added at full speed, in radians.
    pub widen: f32,
    /// Share of the boom added at full speed.
    pub pull: f32,
    /// How much of a third person view this is, from 0 to 1.
    pub third: f32,
}

impl Shot {
    /// Returns the shot `t` of the way from `a` to `b`.
    fn mix(a: &Shot, b: &Shot, t: f32) -> Shot {
        Shot {
            seat: a.seat.lerp(b.seat, t),
            boom: a.boom.lerp(b.boom, t),
            lag: a.lag.lerp(b.lag, t),
            stiff: a.stiff.lerp(b.stiff, t),
            turn: a.turn.lerp(b.turn, t),
            bank: a.bank.lerp(b.bank, t),
            lead: a.lead.lerp(b.lead, t),
            widen: a.widen.lerp(b.widen, t),
            pull: a.pull.lerp(b.pull, t),
            third: a.third.lerp(b.third, t),
        }
    }

    /// Returns the shot of `target` when the view is `person` of the way to third person and
    /// the body `flight` of the way to flying.
    pub(super) fn of(target: &Target, person: f32, flight: f32) -> Shot {
        let pair = |third: bool| match target.kind {
            Kind::Ship => ship(third, target),
            Kind::OnFoot | Kind::Flying => Shot::mix(&foot(third), &fly(third), flight),
        };
        Shot::mix(&pair(false), &pair(true), person)
    }
}

/// Returns the first person shot of a body: the eyes on a head that turns at the neck, with
/// nothing automatic.
fn eyes() -> Shot {
    Shot {
        seat: Vec3::Y * NECK,
        boom: FACE,
        lag: 0.0,
        stiff: 14.0,
        turn: 30.0,
        bank: 0.0,
        lead: 0.0,
        widen: 0.0,
        pull: 0.0,
        third: 0.0,
    }
}

/// Returns the shot of a body on the ground: its eyes, or an orbit over the right shoulder.
fn foot(third: bool) -> Shot {
    if !third {
        return eyes();
    }
    Shot {
        seat: Vec3::Y * SHOULDERS,
        boom: Vec3::new(0.45, 0.15, 3.6),
        lag: 0.5,
        stiff: 9.0,
        third: 1.0,
        ..eyes()
    }
}

/// Returns the shot of a flying body: the horizon banks and the view widens, and in third
/// person the camera sits farther back and higher, lags more and looks ahead.
fn fly(third: bool) -> Shot {
    if !third {
        return Shot {
            bank: 0.5,
            widen: 0.21,
            ..eyes()
        };
    }
    Shot {
        seat: Vec3::Y * SHOULDERS,
        boom: Vec3::new(0.3, 0.9, 6.0),
        lag: 1.2,
        stiff: 5.0,
        bank: 0.35,
        lead: 0.3,
        widen: 0.25,
        pull: 0.15,
        third: 1.0,
        ..eyes()
    }
}

/// Returns the shot of the ship: the pilot seat with a slight lag behind the rotation, or a
/// chase camera sized from the hull that lags more and only trails low in an atmosphere.
fn ship(third: bool, target: &Target) -> Shot {
    let near = target.ground.length().min(1.0);
    if !third {
        return Shot {
            seat: SEAT * target.radius,
            boom: Vec3::ZERO,
            turn: 14.0,
            widen: 0.1,
            ..eyes()
        };
    }
    Shot {
        seat: Vec3::ZERO,
        boom: CHASE * target.radius,
        lag: 0.25 * target.radius * near,
        stiff: 4.0,
        turn: 4.5,
        widen: 0.14,
        pull: 0.25,
        third: 1.0,
        ..eyes()
    }
}
