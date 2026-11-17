//! Universe time: the one clock that orbits, planet spin and saves read.

use bevy::prelude::*;

/// Seconds of universe time since the world was created.
///
/// Planet positions and rotations are pure functions of this value.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Clock {
    /// Elapsed universe time in seconds.
    pub seconds: f64,
}

/// Moves universe time forward by the frame time.
fn advance(time: Res<Time>, mut clock: ResMut<Clock>) {
    clock.seconds += time.delta_secs_f64();
}

/// Registers the universe clock.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Clock>().add_systems(Update, advance);
}
