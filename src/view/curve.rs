//! The pitch curve: one number from looking fully down to fully up that drives the length of
//! the boom, the height of the camera and the field of view together, so the framing holds.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::io::{Heading, Kind};
use super::shot::Shot;
use super::smooth::wrap;
use super::tune::{
    COCKPIT_TURN, COCKPIT_UP, FADE_BAND, INDOOR_DROP, INDOOR_SHORTEN, MAX_FOV, PITCH_LIMIT,
    TILT_DROP, TILT_FOV, TILT_REACH,
};
use crate::render::smoothstep;

/// What bends a shot besides the blends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Bend {
    /// The pitch curve, from [`tilt`].
    pub tilt: f32,
    /// How far indoors the rig is, from 0 to 1.
    pub indoor: f32,
    /// Zoom of the boom.
    pub zoom: f32,
    /// How much speed widens the view.
    pub rush: f32,
}

/// Returns the pitch curve at `pitch`: -1 looking fully down, 0 level, 1 looking fully up, flat at both ends.
///
/// A view past the vertical, on its back in a loop, reads as the same angle from the other
/// horizon, so the curve has no step anywhere on the way round.
pub(super) fn tilt(pitch: f32) -> f32 {
    let folded = match pitch.abs() > FRAC_PI_2 {
        true => pitch.signum() * PI - pitch,
        false => pitch,
    };
    ((folded / PITCH_LIMIT).clamp(-1.0, 1.0) * FRAC_PI_2).sin()
}

/// Returns the pivot of `shot` in the axes of the target: lower indoors.
pub(super) fn seat(shot: &Shot, bend: &Bend) -> Vec3 {
    shot.seat - Vec3::Y * INDOOR_DROP * bend.indoor * shot.third
}

/// Returns the boom of `shot` in the axes the camera looks along.
///
/// Looking up shortens it and drops the camera; looking down lengthens it and raises the
/// camera. Indoors it is shorter, lower and closer to the body. Speed stretches it. All of that
/// applies by the share of third person in the shot: the eyes of a first person view keep their
/// place on the head whatever the zoom.
pub(super) fn boom(shot: &Shot, bend: &Bend) -> Vec3 {
    let third = shot.third;
    let scale = 1.0 + (bend.zoom * (1.0 + shot.pull * bend.rush) - 1.0) * third;
    let reach = (1.0 - INDOOR_SHORTEN * bend.indoor) * (1.0 - TILT_REACH * bend.tilt);
    Vec3::new(
        shot.boom.x * (1.0 - 0.5 * bend.indoor),
        shot.boom.y * (1.0 - bend.indoor * third) - TILT_DROP * bend.tilt * third,
        shot.boom.z * (1.0 + (reach - 1.0) * third),
    ) * scale
}

/// Returns the field of view of `shot` over the `base` setting: wider when looking up from close
/// by, narrower when looking down from afar, and wider with speed.
pub(super) fn fov(base: f32, shot: &Shot, bend: &Bend) -> f32 {
    let framed = base * (1.0 + TILT_FOV * bend.tilt * shot.third);
    (framed + shot.widen * bend.rush).min(MAX_FOV.max(base))
}

/// Returns how much of the body to draw in a view `third` of the way to third person, with the
/// camera `head` metres from the centre of the head: none of its head at the eyes, on foot and
/// in the pilot seat alike, and all of it in third person, however close a wall pushes the
/// camera, unless the camera is inside the head.
pub(super) fn fade(third: f32, head: f32) -> f32 {
    third * smoothstep(FADE_BAND.0, FADE_BAND.1, head)
}

/// Returns the look angles `heading` kept where the head of a pilot can turn in the seat, for
/// a view `third` of the way to third person. At the eyes the head turns as far as
/// [`COCKPIT_TURN`] to each side and looks up as far as [`COCKPIT_UP`], so the view never goes
/// through the headrest; the limits open with the view, and the chase camera orbits freely.
pub(super) fn head(kind: Kind, heading: Heading, third: f32) -> Heading {
    if kind != Kind::Ship || third >= 1.0 {
        return heading;
    }
    let side = COCKPIT_TURN.lerp(PI, third);
    Heading {
        yaw: wrap(heading.yaw).clamp(-side, side),
        pitch: heading.pitch.min(COCKPIT_UP.lerp(PITCH_LIMIT, third)),
    }
}
