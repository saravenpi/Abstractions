//! Keeping the view free: the cast from the pivot to the camera, the whiskers and the easing.
//!
//! The camera pulls in fast when something blocks the view and eases back out slowly. The
//! whiskers feel for walls beside the boom so that it shortens before a wall cuts it. Narrow
//! obstacles are stepped over, so a post or a branch does not make the camera twitch.

use bevy::prelude::*;

use super::io::Occluder;
use super::smooth::closed;
use super::tune::{
    EASE_OUT, EASE_SPEED, PULL_IN, RADIUS, SKIN, THIN_HOP, THIN_PASS, THIN_SHIFT, THIN_SKIPS,
    WHISKER_MIN, WHISKERS,
};

/// A path the camera sphere is cast along.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Probe {
    /// Where the sphere starts.
    pub origin: Vec3,
    /// Unit direction it travels in.
    pub dir: Vec3,
    /// How far it should travel.
    pub max: f32,
    /// The up of the target, which the whiskers fan around.
    pub up: Vec3,
}

impl Probe {
    /// Returns the path from `origin` to the end of `swing`, or `None` when `swing` has no length.
    pub(super) fn along(origin: Vec3, swing: Vec3, up: Vec3) -> Option<Self> {
        let dir = swing.try_normalize()?;
        Some(Self {
            origin,
            dir,
            max: swing.length(),
            up,
        })
    }
}

/// Returns true when what the sphere touched after `hit` metres is narrow, across or along
/// the up: both spheres shifted far to either side of it pass, at least one shifted near
/// passes, and the path is free again a short way behind it.
///
/// Shifted spheres see a wide surface where the first one did, however flat the angle, so
/// floors, walls and door frames are never taken for narrow.
fn narrow(world: &impl Occluder, probe: &Probe, hit: f32) -> bool {
    let side = probe
        .dir
        .cross(probe.up)
        .try_normalize()
        .unwrap_or_else(|| probe.dir.any_orthonormal_vector());
    let free = |axis: Vec3, shift: f32| {
        let origin = probe.origin + axis * shift;
        world
            .cast(origin, probe.dir, hit + THIN_PASS, RADIUS)
            .is_none()
    };
    let (near, far) = THIN_SHIFT;
    let slim =
        |axis: Vec3| free(axis, -far) && free(axis, far) && (free(axis, -near) || free(axis, near));
    let beyond = probe.origin + probe.dir * (hit + THIN_HOP);
    let shallow = world.cast(beyond, probe.dir, SKIN, RADIUS).is_none();
    shallow && (slim(side) || slim(side.cross(probe.dir)))
}

/// Returns how far the camera sphere travels along `probe`, stepping over narrow obstacles.
pub(super) fn reach(world: &impl Occluder, probe: &Probe) -> f32 {
    let mut from = 0.0;
    for skip in 0..=THIN_SKIPS {
        let rest = Probe {
            origin: probe.origin + probe.dir * from,
            max: probe.max - from,
            ..*probe
        };
        let Some(hit) = world.cast(rest.origin, rest.dir, rest.max, RADIUS) else {
            return probe.max;
        };
        if skip == THIN_SKIPS || !narrow(world, &rest, hit) {
            return (from + hit - SKIN).max(0.0);
        }
        from += hit + THIN_HOP;
        if from >= probe.max {
            return probe.max;
        }
    }
    probe.max
}

/// Returns the length the whiskers advise for the boom along `probe`.
///
/// A whisker that touches something asks the boom to shorten part of the way to the touch,
/// less for the outer whiskers: the boom is already shorter when the wall reaches it.
pub(super) fn feel(world: &impl Occluder, probe: &Probe) -> f32 {
    if probe.max < WHISKER_MIN {
        return probe.max;
    }
    let advice = |(angle, slack): &(f32, f32)| {
        let dir = Quat::from_axis_angle(probe.up, *angle) * probe.dir;
        let free = reach(world, &Probe { dir, ..*probe });
        free + (probe.max - free) * slack
    };
    WHISKERS.iter().map(advice).fold(probe.max, f32::min)
}

/// Returns how much shorter than asked the boom is after `dt` seconds, from `short` toward `goal`.
///
/// Pulling in closes the gap at [`PULL_IN`]. Easing out closes it at [`EASE_OUT`] and never
/// faster than [`EASE_SPEED`].
pub(super) fn ease(short: f32, goal: f32, dt: f32) -> f32 {
    if goal >= short {
        return short + (goal - short) * closed(PULL_IN, dt);
    }
    let step = (short - goal) * closed(EASE_OUT, dt);
    short - step.min(EASE_SPEED * dt)
}

/// Returns the pivot of the rig: `seat` away from `origin`, or as far that way as the camera sphere can go.
///
/// The cast starts at the centre of the target, which is free, so the pivot is never inside geometry.
pub(super) fn pivot(world: &impl Occluder, origin: Vec3, seat: Vec3) -> Vec3 {
    let Some(dir) = seat.try_normalize() else {
        return origin;
    };
    let length = seat.length();
    let free = world.cast(origin, dir, length, RADIUS);
    origin + dir * free.map_or(length, |hit| (hit - SKIN).max(0.0))
}
