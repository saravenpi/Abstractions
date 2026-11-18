//! The aim of a body on its wings: the keys always move it, the view only while it steers.
//!
//! The aim is the direction the wings are asked to fly along, see `GearInput::aim`. The caller
//! keeps a [`Stick`] between steps and passes it to [`steer`] with what the hands do. While the
//! steer button is held the aim is the view. Let go, the aim stays where it was and the view
//! looks around freely. The keys work like a stick on top in both cases: they turn the aim, and
//! the view with it, so the camera stays behind the body.

use bevy::prelude::*;

use super::aim::wrap;
use super::path::path_offset;
use super::state::FlightState;
use super::tune::{LEASH, LEVEL_TIME, STICK_RATE};

/// What the hands do during one step on the wings.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Hands {
    /// Pitch stick from -1 (nose down) to 1 (nose up).
    pub pitch: f32,
    /// Roll stick from -1 (left) to 1 (right).
    pub roll: f32,
    /// Whether the steer button is held, so that the view aims the flight.
    pub held: bool,
    /// Heading and pitch the mouse turns the view by during this step, in radians.
    pub mouse: Vec2,
    /// Heading and pitch of the view before this step, in radians.
    pub view: Vec2,
}

/// The aim of a body on its wings, kept between steps.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Stick {
    /// Heading and pitch of the aim, in radians. The pitch may pass the vertical.
    pub aim: Vec2,
    /// Seconds the heading of the aim still rides the flight path for: [`LEVEL_TIME`] while
    /// the roll stick is held, running down once it is let go.
    riding: f32,
}

impl Stick {
    /// Returns a stick at rest on `aim`.
    pub fn at(aim: Vec2) -> Self {
        Self { aim, riding: 0.0 }
    }
}

/// The stick after one step and how far the view turned with it.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Steered {
    /// The stick to keep for the next step.
    pub stick: Stick,
    /// Heading and pitch the view turns by during this step: the mouse and the keys.
    pub turn: Vec2,
}

/// Returns `turn` cut so that a pitch at `from` is not led farther than [`LEASH`] from the
/// flight frame of `state`. It is one way: a pitch already farther is left where it is and may
/// still come back.
fn leashed(state: &FlightState, from: f32, turn: f32) -> f32 {
    let was = wrap(from - state.pitch);
    (was + turn).clamp(was.min(-LEASH), was.max(LEASH)) - was
}

/// Returns the turn the keys give the aim of `stick` in `dt` seconds.
///
/// The pitch stick turns it up or down at [`STICK_RATE`]. The roll stick banks the wings in
/// the gear itself; here it carries the heading of the aim onto the flight path the banked
/// lift turns, no faster than [`STICK_RATE`], so the aim neither runs ahead of the turn nor
/// calls the body back out of it. It goes on riding the path while the wings come level
/// after the key is let go: an aim left where the key went would be overshot by the end of
/// the turn, and the wings would roll the other way to come back to it.
fn keys(stick: &Stick, state: &FlightState, hands: &Hands, dt: f32) -> Vec2 {
    let most = STICK_RATE * dt;
    let carried = match hands.roll != 0.0 || stick.riding > 0.0 {
        true => path_offset(state, stick.aim.x),
        false => 0.0,
    };
    Vec2::new(
        carried.clamp(-most, most),
        hands.pitch.clamp(-1.0, 1.0) * most,
    )
}

/// Returns the turn of the view, and the aim that was at `aim`, for the turn `asked` of both.
///
/// With the steer button up the mouse turns the view alone. With it held the mouse turns both,
/// and an aim that is not on the view goes to it at [`STICK_RATE`], never in one step: once
/// within reach it is the view itself, to the last bit. Nothing leads the aim farther in pitch
/// than [`LEASH`] from the flight frame, nor a view that steers.
fn turns(aim: Vec2, state: &FlightState, hands: &Hands, (asked, dt): (Vec2, f32)) -> (Vec2, Vec2) {
    if !hands.held {
        let turn = Vec2::new(asked.x, leashed(state, aim.y, asked.y));
        return (turn + hands.mouse, aim + turn);
    }
    let asked = asked + hands.mouse;
    let seen = Vec2::new(asked.x, leashed(state, hands.view.y, asked.y));
    let view = hands.view + seen;
    let led = aim + Vec2::new(asked.x, leashed(state, aim.y, asked.y));
    let gap = Vec2::new(wrap(view.x - led.x), wrap(view.y - led.y));
    let chase = gap.clamp_length_max(STICK_RATE * dt);
    let chase = Vec2::new(chase.x, leashed(state, led.y, chase.y));
    match chase == gap {
        true => (seen, view),
        false => (seen, led + chase),
    }
}

/// Returns `stick` after `dt` seconds of `hands` on a body in `state`, whose velocity is the
/// one through the air, see `Gear::in_air`, and the turn of the view. The keys turn the aim
/// and the view alike; the mouse steers only while the steer button is held, see [`turns`].
pub fn steer(stick: &Stick, state: &FlightState, hands: &Hands, dt: f32) -> Steered {
    let asked = keys(stick, state, hands, dt);
    let (turn, aim) = turns(stick.aim, state, hands, (asked, dt));
    let riding = match hands.roll != 0.0 {
        true => LEVEL_TIME,
        false => (stick.riding - dt).max(0.0),
    };
    Steered {
        stick: Stick { aim, riding },
        turn,
    }
}
