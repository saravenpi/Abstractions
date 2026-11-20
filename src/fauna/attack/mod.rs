//! Attacks on and by animals. For now only the harm the robot's blows do to them.

mod harm;
#[cfg(test)]
mod tests_harm;

use bevy::prelude::*;

pub use harm::Dead;

use crate::screens::playing;

/// Registers the harm of blows: before the scan, so a kill leaves the scan's wants at once.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        harm::harm
            .run_if(on_message::<crate::avatar::strike::Hit>)
            .run_if(playing)
            .before(super::scan::scan),
    );
}
