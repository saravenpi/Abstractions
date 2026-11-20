//! Flight gear: the wingsuit and the boot boosters.
//!
//! The flight model is pure: [`step`] takes a [`FlightState`] and a [`GearInput`] and returns
//! the next state. It works in the upright frame of the body, where +Y is the local up, and
//! takes gravity and air density as inputs, so it flies the same way on every planet and thins
//! out with altitude. It does not own the physics body: the player controller calls
//! [`Gear::drive`], moves the body by the returned velocity with move and slide, and hands the
//! slid velocity back through [`Gear::collide`].
//!
//! - Boots alone brake a fall and cannot gain height.
//! - The wingsuit alone glides and always loses energy.
//! - Both together fly.
//! - Under a liquid the boots jet the body along the view, see `jet`.
//!
//! On the wings the keys ask for rates in the frame of the body, not an attitude (see
//! `command`): the roll keys roll it on round its nose, the pitch keys pull toward its back or
//! push toward its belly as hard as the wing may, and the yaw keys turn the path flat. Held,
//! they roll the body over and loop it when it has the speed for it. Let go, the wings come
//! level on the best glide. The view steers inside fixed attitude limits while the steer
//! button is held. Folded, the body faces the view on foot and its flight path in a fall.
//!
//! The effects in `fx` read the [`Gear`] component and publish [`FlightFx`] for the camera,
//! the audio and the companion.

mod aero;
mod aim;
mod assist;
mod boots;
mod command;
mod contact;
mod fx;
mod input;
mod jet;
mod mode;
mod modes;
mod path;
mod roll;
mod servo;
mod state;
mod step;
pub mod tune;

#[cfg(test)]
mod tests;

pub use boots::descent_target;
pub use contact::{deflect, slid};
pub use fx::{FlightFx, GearSet, Wake};
pub use input::{GearInput, air_density};
pub use jet::{boot_power, hull_drag, jet, jetting, liquid_density, pump};
pub use mode::Mode;
pub use state::{FlightState, Servo};
pub use step::step;

use bevy::prelude::*;

use crate::effects::Churn;

/// The flight gear of a body: its state and the input of its last step.
///
/// Both pieces of equipment are owned from the start: every [`crate::player::Player`] gets one.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
#[require(Churn)]
pub struct Gear {
    /// The flight state after the last step.
    pub state: FlightState,
    /// The input of the last step, kept for the effects.
    pub input: GearInput,
}

impl Gear {
    /// Steps the gear by `dt` seconds with `input`.
    ///
    /// `velocity` is the controller's own velocity in the upright frame. It is taken over while
    /// the gear is off and ignored while it drives.
    pub fn drive(&mut self, velocity: Vec3, input: GearInput, dt: f32) {
        if !self.state.mode.drives() {
            self.state.velocity = velocity;
        }
        self.input = input;
        self.state = step(&self.state, &input, dt);
    }

    /// Returns the state with its velocity taken through the air of the last step instead of
    /// over the ground. The wings turn that one, so the stick reads the flight path from it:
    /// in a cross wind the two paths head different ways.
    pub fn in_air(&self) -> FlightState {
        let wind = step::airflow(self.state.mode, &self.input);
        FlightState {
            velocity: self.state.velocity - wind,
            ..self.state
        }
    }

    /// Takes the velocity the physics left after move and slide, in the upright frame.
    pub fn collide(&mut self, moved: Vec3) {
        self.state = slid(&self.state, moved);
    }
}

/// Registers the gear systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(fx::plugin);
}
