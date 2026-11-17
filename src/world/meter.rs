//! A frame-time recorder for the main-thread systems of the streaming chain.

use std::time::Instant;

use bevy::prelude::*;

/// The steps of the streaming chain that are timed, in the order they run.
pub const STEPS: [&str; 10] = [
    "planet", "face", "far", "grow", "stream", "collect", "refine", "show", "place", "sea",
];

/// What one step cost since the last reset.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Cost {
    /// Longest single run in milliseconds.
    pub worst: f32,
    /// Sum of every run in milliseconds.
    pub total: f32,
    /// Number of runs.
    pub runs: u32,
}

/// Main-thread time of each step of the streaming chain, command application included.
///
/// Headless runs read it to check that no step exceeds its budget, then call [`Meter::reset`].
#[derive(Resource, Debug, Clone)]
pub struct Meter {
    last: Instant,
    /// The cost of each step, in the order of [`STEPS`].
    pub costs: [Cost; STEPS.len()],
}

impl Default for Meter {
    /// Returns a meter with nothing measured.
    fn default() -> Self {
        Self {
            last: Instant::now(),
            costs: [Cost::default(); STEPS.len()],
        }
    }
}

impl Meter {
    /// Forgets every measure.
    pub fn reset(&mut self) {
        self.costs = [Cost::default(); STEPS.len()];
    }

    /// Returns the step that had the longest single run, with its cost.
    pub fn worst(&self) -> (&'static str, Cost) {
        let by_worst = |a: &(&str, Cost), b: &(&str, Cost)| a.1.worst.total_cmp(&b.1.worst);
        let all = STEPS.into_iter().zip(self.costs);
        all.max_by(by_worst).unwrap_or(("none", Cost::default()))
    }
}

/// Starts the clock of the chain.
pub fn start(mut meter: ResMut<Meter>) {
    meter.last = Instant::now();
}

/// Charges the time since the previous lap to step `STEP` of [`STEPS`].
pub fn lap<const STEP: usize>(mut meter: ResMut<Meter>) {
    let now = Instant::now();
    let millis = (now - meter.last).as_secs_f32() * 1000.0;
    meter.last = now;
    let cost = &mut meter.costs[STEP];
    cost.worst = cost.worst.max(millis);
    cost.total += millis;
    cost.runs += 1;
}
