//! The cockpit as the pilot uses it: where the controls and the instruments are in the hull,
//! how far each one is moved by the real state of the ship, and the rules that say so.
//!
//! Everything here is in the axes of the hull: the nose along -Z, the roof along +Y. The
//! meshes of the cabin are built around these places, the parts are moved by these rules and
//! the hands and the feet of the pilot hold the controls these rules give.
//!
//! The controls follow the real input: the stick the pitch, roll and yaw commands, the lever
//! the throttle, the pedals the yaw, the gear lever and the warp switch the state of the gear
//! and of the drive. The instruments read the telemetry the HUD reads; one with nothing true to
//! show goes dark. The displays are small pictures painted on the CPU, and a display is
//! repainted only when its quantised value changes.

mod attitude;
mod canopy;
mod controls;
mod display;
mod faces;
mod fit;
mod font;
mod kit;
mod light;
mod others;
mod paint;
mod part;
mod pixels;
mod readings;
mod remote;
mod screens;
mod shapes;
mod sources;
mod state;
mod strap;
mod tune;
mod work;

#[cfg(test)]
mod stage;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app;
#[cfg(test)]
mod tests_dark;
#[cfg(test)]
mod tests_horizon;
#[cfg(test)]
mod tests_life;
#[cfg(test)]
mod tests_paint;

use bevy::prelude::*;

pub use canopy::Canopy;
pub use controls::Levers;
pub use fit::{Lit, fit};
pub use kit::Kit;
pub use part::Part;
pub use readings::{Dial, Lamp, Readings};
pub use remote::Seen;
pub use screens::{Display, Face, Screen};
pub use state::Cockpit;
pub use tune::{
    BACK, BAYS, CANOPY_OPEN, FLOOR, FOLLOW, GEAR_LEVER, HINGE, LEVER, LEVER_LENGTH, LEVER_SWING,
    PAN, PEDAL, PEDAL_LENGTH, PEDAL_REACH, RAKE, RAKE_TILT, SETTLE, STICK, STICK_LENGTH,
    STICK_REACH, TEXEL, WARP_SWITCH,
};

/// Registers the systems of the cockpit: the local ship is fed from its helm and its
/// instruments, the displays are fitted and the lights rigged, then the parts of every cockpit
/// are moved, the displays repainted, the lights set and the glowing faces dimmed.
pub(super) fn plugin(app: &mut App) {
    let run = (
        sources::feed,
        display::equip.run_if(display::fitted),
        light::rig.run_if(light::fitting),
        work::work,
        canopy::swing,
        display::repaint,
        light::shine,
        light::dim.run_if(any_with_component::<Lit>),
    )
        .chain();
    let run = run.run_if(any_with_component::<Cockpit>);
    app.add_systems(Update, run.in_set(super::ShipSet::Place));
}
