//! The day clock: a twenty minute cycle, ten minutes of day and ten of night.

use bevy::prelude::*;

use crate::saves::ActiveWorld;
use crate::screens::{Screen, playing};

/// Real seconds in one full day and night cycle.
pub const CYCLE_SECONDS: f32 = 1200.0;

/// Phase at the start of a new game, early morning.
pub const START_PHASE: f32 = 0.08;

/// Position in the day cycle, from 0 to 1.
///
/// 0.0 is sunrise, 0.25 noon, 0.5 sunset and 0.75 midnight.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct DayClock {
    /// Current phase of the cycle.
    pub phase: f32,
}

impl Default for DayClock {
    /// Starts in the early morning.
    fn default() -> Self {
        Self { phase: START_PHASE }
    }
}

/// Moves the clock forward by the frame time.
fn advance(time: Res<Time>, mut clock: ResMut<DayClock>) {
    clock.phase = (clock.phase + time.delta_secs() / CYCLE_SECONDS).fract();
}

/// Sets the clock to the saved phase of the active world, or the early morning without one.
fn reset(mut clock: ResMut<DayClock>, active: Option<Res<ActiveWorld>>) {
    let saved = active.map(|active| active.0.save.day_phase);
    clock.phase = saved
        .filter(|phase| phase.is_finite())
        .map_or(START_PHASE, f32::fract);
}

/// Registers the clock, which only runs while the player is in the world.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DayClock>()
        .add_systems(OnEnter(Screen::Loading), reset)
        .add_systems(Update, advance.run_if(playing));
}
