//! Acts: what the hands do with the tool, one step at a time, and the key each act poses.

use bevy::prelude::*;

use super::dir::Dir;
use super::held::Held;
use super::key::{Key, carry, drawn, guard};
use super::path::Path;
use super::tune::{DRAW, GUARD_IN, LEAST_DRAW, RECOVER};

/// What the hands are doing. Every time is seconds since the act began.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Act {
    /// Carrying the tool, ready.
    #[default]
    Ready,
    /// Cocking the tool back before a swing.
    Windup(f32),
    /// Swinging through, at the speed of the build.
    Swing(f32),
    /// Coming back to carrying.
    Recover(f32),
    /// Holding a guard up.
    Guard(f32),
    /// Drawing a spear back to throw it.
    Draw(f32),
    /// Shaken by the kickback of a blow: seconds left, and the share of the path where it landed.
    Stun(f32, f32),
}

/// The strike state of a body, on its root beside [`Held`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Strike {
    /// What the hands do.
    pub act: Act,
    /// The side the strike or the guard is on.
    pub dir: Dir,
    /// True once the swing under way has landed a blow, so it lands one only.
    pub landed: bool,
    /// The grip and the tip of the tool this frame and the one before, in the frame of the body.
    pub blade: [[Vec3; 2]; 2],
}

impl Strike {
    /// Returns the side the guard covers while one is up, for the attacks of animals to read.
    pub fn guarding(&self) -> Option<Dir> {
        matches!(self.act, Act::Guard(_)).then_some(self.dir)
    }

    /// Returns true while the guard was raised less than `window` seconds ago: a blow on its side
    /// now is parried, not only blocked. See [`Held::parry`] for the window of a tool.
    pub fn parries(&self, window: f32) -> bool {
        matches!(self.act, Act::Guard(t) if t < window)
    }
}

/// What the buttons ask this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ask {
    /// The primary button went down.
    pub strike: bool,
    /// The primary button is held.
    pub holding: bool,
    /// The secondary button is held.
    pub guard: bool,
    /// The direction the mouse asks.
    pub dir: Dir,
}

/// How long each part of a swing takes, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    /// From carrying to cocked: 2·arc / ω, stretched by torque over the grip budget.
    pub windup: f32,
    /// Through the path at the speed of the build.
    pub swing: f32,
    /// Back to carrying.
    pub recover: f32,
}

impl Timing {
    /// Returns the timing of `held` along `path`: a swing turns at the build's spin, a thrust moves
    /// at its thrust speed.
    pub fn of(held: &Held, path: &Path) -> Self {
        let speed = match path {
            Path::Sweep { .. } => held.swing.spin,
            Path::Line { .. } => held.swing.thrust,
        };
        Self {
            windup: held.swing.windup,
            swing: path.length() / speed.max(f32::EPSILON),
            recover: RECOVER,
        }
    }
}

/// Returns the act that follows a ready body asked `ask`: a guard, a draw for a thrown tool, else a
/// wind-up.
fn ready(ask: &Ask, thrown: bool) -> Act {
    match (ask.guard, ask.strike, thrown) {
        (true, _, _) => Act::Guard(0.0),
        (false, true, true) => Act::Draw(0.0),
        (false, true, false) => Act::Windup(0.0),
        _ => Act::Ready,
    }
}

/// Returns `same` of the new time `t + dt`, or `then` once that reaches `length`.
fn timed(t: f32, dt: f32, length: f32, (then, same): (Act, fn(f32) -> Act)) -> Act {
    match t + dt >= length {
        true => then,
        false => same(t + dt),
    }
}

/// Returns the act after `act` has run `dt` more seconds under `ask`, and the share of a full draw
/// when a spear is let go this step.
pub fn next(act: Act, ask: &Ask, timing: &Timing, thrown: bool, dt: f32) -> (Act, Option<f32>) {
    let act = match act {
        Act::Ready => ready(ask, thrown),
        Act::Windup(t) => timed(t, dt, timing.windup, (Act::Swing(0.0), Act::Windup)),
        Act::Swing(t) => timed(t, dt, timing.swing, (Act::Recover(0.0), Act::Swing)),
        Act::Recover(t) => timed(t, dt, timing.recover, (Act::Ready, Act::Recover)),
        Act::Stun(left, _) if left <= dt => Act::Recover(0.0),
        Act::Stun(left, at) => Act::Stun(left - dt, at),
        Act::Guard(t) if ask.guard => Act::Guard(t + dt),
        Act::Guard(_) => Act::Ready,
        Act::Draw(t) => return draw(t, ask, dt),
    };
    (act, None)
}

/// Returns the act of a spear drawn for `t` seconds under `ask`: cancelled by the guard, thrown on
/// release when drawn far enough.
fn draw(t: f32, ask: &Ask, dt: f32) -> (Act, Option<f32>) {
    let share = (t / DRAW).min(1.0);
    match (ask.guard, ask.holding) {
        (true, _) => (Act::Ready, None),
        (false, true) => (Act::Draw(t + dt), None),
        (false, false) if share >= LEAST_DRAW => (Act::Recover(0.0), Some(share)),
        (false, false) => (Act::Ready, None),
    }
}

/// Returns the key `act` poses along `path` with `timing`, guarding on the side `dir`.
pub fn key(act: Act, path: &Path, timing: &Timing, dir: Dir) -> Key {
    let share = |t: f32, length: f32| t / length.max(f32::EPSILON);
    match act {
        Act::Ready => carry(),
        Act::Windup(t) => carry().toward(&path.at(0.0), share(t, timing.windup)),
        Act::Swing(t) => path.at(share(t, timing.swing)),
        Act::Recover(t) => path.at(1.0).toward(&carry(), share(t, timing.recover)),
        Act::Guard(t) => carry().toward(&guard(dir), share(t, GUARD_IN)),
        Act::Draw(t) => carry().toward(&drawn(), share(t, DRAW)),
        Act::Stun(_, at) => path.at(at),
    }
}
