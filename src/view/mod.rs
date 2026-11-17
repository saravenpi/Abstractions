//! The camera rig: first person and third person views, on foot, in flight and in the ship.
//!
//! The rig is pure logic behind plain data. [`Rig::step`] reads a [`Moment`] (the [`Target`],
//! the [`Look`] of the player, the [`ViewPrefs`], the field of view setting and the frame
//! time) and anything that implements [`Occluder`], and returns the [`CameraPose`] and the
//! [`BodyFade`]. The systems in this module feed it from the game and move the game camera,
//! once [`Enabled`] is switched on.
//!
//! What it applies, and where that comes from:
//!
//! - The camera is described relative to its target by distance, yaw and pitch, and only
//!   turned into a position at the end. The player's input always wins over anything the rig
//!   does by itself. Occlusion is tested from the target toward the camera, pulls in fast,
//!   eases out slowly and ignores narrow obstacles. A sphere keeps the camera body out of
//!   geometry and the body fades when the camera comes close. From John Nesky's talk on the
//!   camera of Journey, written up at https://weekly-geekly.imtqy.com/articles/272401/index.html
//! - The camera lags a little behind the position of its target and less behind its rotation,
//!   and one pitch curve drives distance and field of view together. From
//!   https://www.unrealengine.com/tech-blog/six-ingredients-for-a-dynamic-third-person-camera
//! - Narrow places get their own handling: an upward cast finds the ceiling and the rig
//!   shortens and lowers indoors, and whiskers shorten the boom before a wall cuts it. From
//!   https://www.gamedeveloper.com/design/third-person-camera-view-in-games-a-record-of-the-most-common-problems-in-modern-games-solutions-taken-from-new-and-retro-games
//!
//! All smoothing is exponential in the frame time, so the view is the same at every frame
//! rate. All state is relative to the target, so a rebase of the render anchor is invisible.

mod aim;
mod basis;
mod blend;
mod curve;
mod io;
mod keys;
mod mode;
mod occlude;
mod rig;
mod shot;
mod smooth;
mod sway;
mod system;
pub mod tune;

#[cfg(test)]
mod tests;

pub use io::{BodyFade, CameraPose, Heading, Kind, Look, Moment, Occluder, Shown, Target};
pub use mode::{ViewMode, ViewPrefs};
pub use rig::Rig;
pub use system::{Cut, Enabled, Unseen, ViewSet, enabled};

use bevy::prelude::*;

/// Registers the view systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(system::plugin);
}
