//! Which mode the gear is in: what the held inputs ask for in the air, and how a touchdown ends.

use bevy::prelude::*;

use super::input::GearInput;
use super::mode::Mode;
use super::state::FlightState;
use super::tune::{
    ROLL_DAMP, ROLL_SINE, ROLL_SINK, ROLL_TIME, SLIDE_DRAG, SLIDE_ENTER, SLIDE_EXIT, SLIDE_G,
};

/// Returns the mode the held inputs ask for in the air.
///
/// A body the gear never took over keeps falling under the controller; one it did take over
/// falls ballistically under the gear until it lands.
fn airborne(mode: Mode, input: &GearInput) -> Mode {
    match (input.deploy, input.boost) {
        (true, true) => Mode::Fly,
        (true, false) => Mode::Glide,
        (false, true) => Mode::BootFall,
        (false, false) if mode == Mode::Off => Mode::Off,
        (false, false) => Mode::Fall,
    }
}

/// Returns how a body arriving on the ground at `velocity` lands, having already lost `impact`
/// metres per second into the surface.
///
/// Steep and hard is a landing roll, shallow and fast a slide, anything slow simply stands.
fn touchdown(velocity: Vec3, impact: f32) -> Mode {
    let sink = impact.max(-velocity.y);
    let speed = velocity.length_squared().max(sink * sink).sqrt();
    if sink > ROLL_SINK && sink > speed * ROLL_SINE {
        Mode::Roll
    } else if velocity.with_y(0.0).length() > SLIDE_ENTER {
        Mode::Slide
    } else {
        Mode::Off
    }
}

/// Returns the mode of a body on the ground.
fn landed(state: &FlightState) -> Mode {
    let fast = state.velocity.with_y(0.0).length();
    match state.mode {
        Mode::Off => Mode::Off,
        Mode::Roll if state.servo.clock < ROLL_TIME => Mode::Roll,
        Mode::Roll | Mode::Slide if fast > SLIDE_EXIT => Mode::Slide,
        Mode::Roll | Mode::Slide => Mode::Off,
        _ => touchdown(state.velocity, state.servo.impact),
    }
}

/// Returns the mode for the next step.
///
/// Water always folds the gear and hands over to swimming.
pub(super) fn next(state: &FlightState, input: &GearInput) -> Mode {
    if input.in_water {
        Mode::Off
    } else if input.grounded {
        landed(state)
    } else {
        airborne(state.mode, input)
    }
}

/// Returns the velocity of a sliding body after `h` seconds: friction that grows with speed.
pub(super) fn slide(velocity: Vec3, gravity: f32, h: f32) -> Vec3 {
    let speed = velocity.length();
    let shed = (SLIDE_G * gravity + SLIDE_DRAG * speed) * h;
    velocity * ((speed - shed).max(0.0) / speed.max(1e-3))
}

/// Returns the velocity of a rolling body after `h` seconds: it sheds speed fast but smoothly.
pub(super) fn roll(velocity: Vec3, h: f32) -> Vec3 {
    velocity * (-ROLL_DAMP * h).exp()
}
