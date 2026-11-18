//! A pilot for the test flights: hands on the stick law, and the aim and the view they keep.

use bevy::prelude::*;

use super::{DT, Flight};
use crate::gear::GearInput;
use crate::gear::stick::{Hands, Stick, steer};

/// A flight with a pilot: the aim is kept between frames and steered the way the game does.
#[derive(Debug, Clone, Copy)]
pub(super) struct Pilot {
    /// The body the pilot flies.
    pub(super) flight: Flight,
    /// The stick the pilot keeps, with the aim.
    pub(super) stick: Stick,
    /// Heading and pitch of the view, in radians.
    pub(super) view: Vec2,
}

impl Pilot {
    /// Returns a pilot of `flight` who aims and looks dead ahead, `pitch` radians above the
    /// horizon.
    pub(super) fn on(flight: Flight, pitch: f32) -> Self {
        let ahead = Vec2::new(flight.state.servo.yaw, pitch);
        Self {
            flight,
            stick: Stick::at(ahead),
            view: ahead,
        }
    }

    /// Flies one frame of `dt` seconds with `hands`, in the air and with the switches of
    /// `input`. The view of `hands` is the one the pilot keeps.
    pub(super) fn tick(&mut self, hands: &Hands, input: &GearInput, dt: f32) {
        let hands = Hands {
            view: self.view,
            ..*hands
        };
        let steered = steer(&self.stick, &self.flight.state, &hands, dt);
        (self.stick, self.view) = (steered.stick, self.view + steered.turn);
        let aim = self.stick.aim;
        let look = Quat::from_euler(EulerRot::YXZ, aim.x, aim.y, 0.0) * Vec3::NEG_Z;
        let told = GearInput {
            pitch: hands.pitch,
            roll: hands.roll,
            aim: Some(aim),
            look,
            ..*input
        };
        self.flight.tick(&told, dt);
    }

    /// Flies for `seconds` at the test frame time.
    pub(super) fn run(&mut self, hands: &Hands, input: &GearInput, seconds: f32) {
        for _ in 0..(seconds / DT).round() as u32 {
            self.tick(hands, input, DT);
        }
    }
}

/// Returns hands that hold the pitch and roll keys at `pitch` and `roll`, the steer button up
/// and the mouse still.
pub(super) fn keys(pitch: f32, roll: f32) -> Hands {
    Hands {
        pitch,
        roll,
        ..Hands::default()
    }
}
