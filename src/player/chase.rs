//! The chase view on the wings: how the view follows a body that loops and rolls over.
//!
//! The gear reads the attitude as a heading and a pitch that may pass the vertical, so one nose
//! has two readings: one with the view upright and one with the view on its back. The view
//! follows whichever reading is nearest it, so it goes over the top of a loop with the body and
//! never snaps a half turn, and it rights itself once the body faces the sky again.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::wrap;
use crate::gear::FlightState;

/// Seconds the steer button must have been up and the mouse left alone on the wings before the
/// view drifts back behind the body: long enough to look around.
const SETTLE: f32 = 2.5;

/// Rate per second at which the view drifts back behind a body on its wings.
const RETURN: f32 = 4.0;

/// Fastest the view drifts back, in radians per second: a view left far round swings back
/// steadily instead of whipping a third person camera across.
const SWING: f32 = 2.5;

/// Largest turn in radians, of the heading and of the pitch each, the view is carried by in one
/// frame: the most a body turns in a frame at 30 frames a second. A reading that jumps, as the
/// heading of a body falling straight down, never whips the camera round.
pub(super) const CARRY: f32 = 0.2;

/// Gap in radians, heading and pitch together, at which a view that rights itself is upright.
const RIGHTED: f32 = 0.05;

/// What the chase view of the local body on its wings keeps between frames.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct WingView {
    /// Nose heading and frame pitch of the body on the last frame, while on the wings.
    last: Option<Vec2>,
    /// Seconds the steer button has been up and the mouse left alone.
    pub(super) idle: f32,
    /// Whether the view follows the body read the other way round, see [`other`].
    over: bool,
    /// Whether the view is righting itself, see [`chased`].
    rights: bool,
}

/// Returns the heading and pitch `body` reads the other way round: the heading turned by a
/// half turn and the pitch mirrored across the vertical. The nose points the same way, and a
/// view along it is on its back where the first is upright.
fn other(body: Vec2) -> Vec2 {
    Vec2::new(wrap(body.x + PI), PI.copysign(body.y) - body.y)
}

/// Returns how far apart the views `a` and `b` are, heading and pitch together.
fn apart(a: Vec2, b: Vec2) -> f32 {
    wrap(a.x - b.x).abs() + wrap(a.y - b.y).abs()
}

/// Returns whether [`other`] of `body` is nearer the view at `view` than `body` itself.
fn flipped(body: Vec2, view: Vec2) -> bool {
    apart(other(body), view) < apart(body, view)
}

/// Returns `body`, or [`other`] of it when `flip`.
fn read(body: Vec2, flip: bool) -> Vec2 {
    match flip {
        true => other(body),
        false => body,
    }
}

/// Returns how far the body turned since the last frame as the view at `view` sees it, and
/// keeps `body` for the next frame. Both frames are read the way the body is read nearest the
/// view, so the turn stays small however the gear reads the body, and never more than
/// [`CARRY`].
pub(super) fn carried(wing: &mut WingView, body: Vec2, view: Vec2) -> Vec2 {
    let Some(last) = wing.last.replace(body) else {
        return Vec2::ZERO;
    };
    wing.over = flipped(body, view);
    let (now, was) = (read(body, wing.over), read(last, wing.over));
    let turn = Vec2::new(wrap(now.x - was.x), wrap(now.y - was.y));
    turn.clamp(Vec2::splat(-CARRY), Vec2::splat(CARRY))
}

/// Returns the reading of a body in `state`, read `body` by the gear, that the view at `view`
/// drifts toward.
///
/// It is the reading nearest the view, so a view that went over the top of a loop with the
/// body goes on round with it. A view on its back behind a body whose back faces the sky, as
/// after a roll out at the top of a loop, rights itself: it goes to the upright reading at
/// once, and on until it is there.
pub(super) fn chased(wing: &mut WingView, state: &FlightState, body: Vec2, view: Vec2) -> Vec2 {
    let upright = read(body, body.y.cos() < 0.0);
    let sky = (state.frame() * Vec3::Y).y > 0.0;
    let started = wing.rights || view.y.cos() < 0.0;
    wing.rights = sky && started && apart(upright, view) > RIGHTED;
    match wing.rights {
        true => upright,
        false => read(body, flipped(body, view)),
    }
}

/// Returns the turn that drifts a view at `view` onto `goal` in `dt` seconds, at [`RETURN`] and
/// never faster than [`SWING`]. Nothing drifts before the view has been left alone for
/// [`SETTLE`], unless it rights itself.
pub(super) fn drift(goal: Vec2, view: Vec2, wing: &WingView, dt: f32) -> Vec2 {
    if wing.idle < SETTLE && !wing.rights {
        return Vec2::ZERO;
    }
    let gap = Vec2::new(wrap(goal.x - view.x), wrap(goal.y - view.y));
    (gap * (1.0 - (-RETURN * dt).exp())).clamp_length_max(SWING * dt)
}
