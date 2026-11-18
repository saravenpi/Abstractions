//! The fixed simulation step: the rules that must agree between players run in [`SimSet`].
//!
//! The step rides on Bevy's fixed clock without changing its rate, which avian also runs on:
//! a fixed tick that completes a step of `1 / STEPS_PER_SECOND` seconds runs the set, any other
//! tick skips it. The fixed rate must stay at or above [`STEPS_PER_SECOND`]; Bevy's is 64 Hz.

use bevy::prelude::*;

use super::STEPS_PER_SECOND;

/// The rules that must agree between players, in `FixedUpdate`, once per simulation step.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimSet;

/// Fixed time not yet spent on simulation steps, and whether this fixed tick runs one.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
struct Owed {
    /// Seconds of fixed time since the last step.
    seconds: f64,
    /// Whether the fixed tick in progress runs a step.
    due: bool,
}

/// Returns the time owed after a fixed tick of `dt` seconds, and whether that tick runs a step.
fn advance(owed: f64, dt: f64) -> (f64, bool) {
    let length = 1.0 / STEPS_PER_SECOND;
    let total = owed + dt;
    match total + 1.0e-9 >= length {
        true => (total - length, true),
        false => (total, false),
    }
}

/// Counts the fixed tick and decides whether it runs a simulation step.
fn tick(time: Res<Time>, mut owed: ResMut<Owed>) {
    let (seconds, due) = advance(owed.seconds, time.delta_secs_f64());
    *owed = Owed { seconds, due };
}

/// Registers the step: the count runs first in every fixed tick, the set only on a step.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Owed>()
        .configure_sets(FixedUpdate, SimSet.run_if(|owed: Res<Owed>| owed.due))
        .add_systems(FixedUpdate, tick.before(SimSet));
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;

    /// Counts the runs of the simulation set.
    #[derive(Resource, Default)]
    struct Runs(u32);

    /// A second of Bevy's 64 Hz fixed ticks runs exactly 30 steps, never two in one tick.
    #[test]
    fn a_second_of_fixed_ticks_runs_thirty_steps() {
        let (mut owed, mut steps) = (0.0, 0);
        for _ in 0..64 {
            let (left, due) = advance(owed, 1.0 / 64.0);
            (owed, steps) = (left, steps + u32::from(due));
            assert!(owed < 1.0 / STEPS_PER_SECOND);
        }
        assert_eq!(steps, 30);
    }

    /// In an app the set runs once per step of game time, whatever the frame length.
    #[test]
    fn the_set_runs_at_the_step_rate_in_an_app() {
        let mut app = App::new();
        let frame = TimeUpdateStrategy::ManualDuration(Duration::from_millis(10));
        app.add_plugins((MinimalPlugins, plugin))
            .insert_resource(frame)
            .init_resource::<Runs>()
            .add_systems(
                FixedUpdate,
                (|mut runs: ResMut<Runs>| runs.0 += 1).in_set(SimSet),
            );
        for _ in 0..301 {
            app.update();
        }
        let runs = app.world().resource::<Runs>().0;
        assert!((89..=90).contains(&runs), "{runs} steps in 3 s");
    }
}
