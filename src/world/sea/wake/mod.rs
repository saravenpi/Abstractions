//! The wake of a source on the sea: the waves it leaves in the surface itself.
//!
//! A source that blows on the sea or skims it lays a [`Packet`] of waves into the ring buffer
//! [`Wakes`] while it moves, or while it hovers. The water shader reads the live entries,
//! displaces the surface and tilts its normals by them; `swell.rs` raises the same height on the
//! CPU, so what floats and the waterline agree with what is drawn. The foam shader reads them
//! too, so the foam rides the waves. Nothing is read when no entry lives.

mod block;
mod field;
mod packet;
mod physics;
mod tune;
mod wakes;

#[cfg(test)]
mod tests_field;
#[cfg(test)]
mod tests_wakes;
#[cfg(test)]
mod tests_wedge;

use bevy::prelude::*;

pub use block::{FoamWakes, SeaWakes};
pub use packet::Packet;
pub use physics::{amplitude, arm_angle, froude, transverse_wavelength, wedge};
pub use tune::{HOVER_WAVELENGTH, MIN_AMPLITUDE, MOVING, PERIOD};
pub use wakes::{Wake, Wakes};

/// Registers the ring buffer and loads the shader module that the water and the foam import as
/// `package::world::sea::wake::field`. An app without shaders, as in a test, loads nothing.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Wakes>();
    if app.world().contains_resource::<Assets<Shader>>() {
        bevy::shader::load_shader_library!(app, "field.wesl");
    }
}
