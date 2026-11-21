//! Keeping what the rig reads within bounds, so that one bad frame of the game never leaves a
//! value that is not a number in its state.
//!
//! A warp between galaxies runs at 3.4e19 m/s. Squared in f32 that is infinite, and an infinite
//! speed made the stride of the head bob not a number for good: the chase camera then sat at
//! the centre of the hull for the rest of the game.

use bevy::prelude::*;

use super::io::{Look, Moment, Target};
use super::mode::ViewPrefs;
use super::tune::{BASE_FOV, DT_RANGE, FASTEST};

/// Returns `value`, or `or` when it is not finite.
fn real(value: f32, or: f32) -> f32 {
    if value.is_finite() { value } else { or }
}

/// Returns `value`, or `or` when one of its parts is not finite.
fn real3(value: Vec3, or: Vec3) -> Vec3 {
    if value.is_finite() { value } else { or }
}

/// Returns `rate` with its direction kept and its size at most [`FASTEST`], measured on its
/// largest part so that a rate whose square is past the range of f32 is tamed too. A rate that
/// is not finite is zero.
pub(super) fn tamed(rate: Vec3) -> Vec3 {
    let rate = real3(rate, Vec3::ZERO);
    let most = rate.abs().max_element();
    match most > FASTEST {
        true => rate / most * FASTEST,
        false => rate,
    }
}

/// Returns `target` with every part finite and its velocity and spin tamed.
fn target(target: &Target) -> Target {
    let base = Target::default();
    Target {
        position: real3(target.position, base.position),
        up: real3(target.up, base.up),
        forward: real3(target.forward, base.forward),
        velocity: tamed(target.velocity),
        spin: tamed(target.spin),
        ground: real3(target.ground, base.ground),
        radius: real(target.radius, base.radius),
        kind: target.kind,
        bank: real(target.bank, base.bank),
        back: real3(target.back, base.back),
    }
}

/// Returns the moment the rig steps on: `moment` with every number finite, a zoom and speed
/// effects that are not numbers at their defaults, and a frame time that is not one at the
/// shortest frame.
pub(super) fn moment(moment: &Moment) -> Moment {
    let (look, prefs, base) = (&moment.look, &moment.prefs, ViewPrefs::default());
    Moment {
        target: target(&moment.target),
        look: Look {
            yaw_delta: real(look.yaw_delta, 0.0),
            pitch_delta: real(look.pitch_delta, 0.0),
            zoom_delta: real(look.zoom_delta, 0.0),
            recentre: look.recentre,
        },
        prefs: ViewPrefs {
            zoom: real(prefs.zoom, base.zoom),
            speed_fx: real(prefs.speed_fx, base.speed_fx),
            ..*prefs
        },
        fov: real(moment.fov, BASE_FOV),
        dt: real(moment.dt, DT_RANGE.0),
    }
}
