//! The day clock: the time of day at the camera, read from where the star stands in the sky.

use bevy::prelude::*;

use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::universe::PlanetSpec;

/// Phase at the start of a new game, early morning.
pub const START_PHASE: f32 = 0.08;

/// Share of a day a wanted phase may lie behind the current one and still count as reached.
pub const PHASE_SLACK: f32 = 1.0e-3;

/// Position in the day cycle, from 0 to 1.
///
/// 0.0 is sunrise, 0.25 noon, 0.5 sunset and 0.75 midnight.
///
/// The sky derives the phase every frame from the angle of the star at the camera. Writing another
/// value asks for that time of day: while the camera stands on a planet, universe time moves
/// forward until the star is there.
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

/// Returns the seconds from one noon to the next on a planet: its turn and its orbit add up.
pub fn solar_day(planet: &PlanetSpec) -> f64 {
    1.0 / (1.0 / planet.day_seconds + 1.0 / planet.orbit_seconds)
}

/// Returns the seconds universe time must move forward for the phase `now` to become `wanted`.
///
/// A wanted phase less than [`PHASE_SLACK`] behind counts as reached, so asking for the current
/// phase again does not skip a whole day.
pub fn catch_up(now: f32, wanted: f32, planet: &PlanetSpec) -> f64 {
    let ahead = (wanted - now).rem_euclid(1.0);
    if ahead > 1.0 - PHASE_SLACK {
        return 0.0;
    }
    f64::from(ahead) * solar_day(planet)
}

/// Whether the phase the day clock holds is only a display value: universe time already says
/// what time it is, so the sky must not move it to reach that phase.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Settled(pub bool);

/// Asks for the saved phase of the active world, or the early morning without one.
///
/// A world resumed on a planet comes with its universe time, which places the star exactly
/// where it was: its phase is settled and asks for nothing. Moving time to reach it would move
/// every planet away from a ship saved in space.
fn reset(
    mut clock: ResMut<DayClock>,
    mut settled: ResMut<Settled>,
    active: Option<Res<ActiveWorld>>,
) {
    let save = active.as_ref().map(|active| &active.0.save);
    settled.0 = save.is_some_and(|save| save.place.is_some());
    clock.phase = save
        .map(|save| save.day_phase)
        .filter(|phase| phase.is_finite())
        .map_or(START_PHASE, f32::fract);
}

/// Registers the clock, which the sun tracking keeps in step with the sky.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<DayClock>()
        .init_resource::<Settled>()
        .add_systems(OnEnter(Screen::Loading), reset);
}
