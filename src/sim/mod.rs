//! The simulation contract that keeps the game ready for more than one player.
//!
//! The world is the seed plus a log of what players changed. Players act through intents, which
//! the rules read instead of the keyboard, so a remote player is the same message from another
//! source. Every rule takes a player by [`PlayerId`]; only the camera, the interface and the
//! input code may assume there is one player.
//!
//! The rules that must agree between players run in [`SimSet`], in `FixedUpdate`, at
//! [`STEPS_PER_SECOND`]: whatever changes the shared world from intents, the seed and the log
//! alone, such as the harvests, crafts and strikes of later versions. A rule there reads
//! intents, the seed and the log, takes the step length, never the frame time, and never reads
//! the keyboard, the camera or the clock of the machine. Nothing runs there yet: the bodies move
//! in `Update`, where avian's spatial queries are valid, and physics keeps avian's own schedule.

mod intent;
mod log;
#[cfg(test)]
mod log_tests;
mod step;

pub use abstractions_net::log::{Change, Changes, Entry, PlayerId};
pub use intent::{Intent, IntentOf};
pub use log::ShareChange;
pub use step::SimSet;

use bevy::prelude::*;

/// Simulation steps per second: the rules that must agree between players run at this rate.
pub const STEPS_PER_SECOND: f64 = 30.0;

/// Registers the intents, the fixed simulation step and the log of first visits.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Changes>()
        .add_message::<IntentOf>()
        .add_plugins((step::plugin, log::plugin));
}
