//! A check of dev builds on the visibility ranges the game hands Bevy.
//!
//! Bevy gives every distinct visibility range its own GPU slot for the whole run of the app and
//! never frees one. Past 65,535 slots every new range shares slot 0: meshes are culled or dithered
//! by the wrong range, and the ground blinks and turns transparent. The game uses a few dozen
//! ranges, so many more is a bug, and a dev build says so once.

use std::collections::HashSet;

use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;

/// Most distinct visibility ranges the game is expected to use in one run of the app.
pub const RANGE_BOUND: usize = 256;

/// Every distinct visibility range seen since the app started, as Bevy counts them.
#[derive(Default)]
pub struct Seen {
    /// The ranges, compared as Bevy compares them.
    ranges: HashSet<VisibilityRange>,
    /// Whether the count already passed [`RANGE_BOUND`] and was told.
    told: bool,
}

impl Seen {
    /// Counts `range` and returns true once, the first time the count passes [`RANGE_BOUND`]. A
    /// range with a NaN margin never equals itself, so it counts anew every time, as in Bevy.
    pub fn note(&mut self, range: &VisibilityRange) -> bool {
        self.ranges.insert(range.clone());
        let over = !self.told && self.ranges.len() > RANGE_BOUND;
        self.told |= over;
        over
    }

    /// Returns how many distinct ranges were seen.
    pub fn count(&self) -> usize {
        self.ranges.len()
    }
}

/// Counts the visibility ranges added or changed this frame and warns once when there are more
/// than [`RANGE_BOUND`] of them in the run, naming the range that passed it.
fn watch(mut seen: Local<Seen>, ranges: Query<&VisibilityRange, Changed<VisibilityRange>>) {
    for range in &ranges {
        if seen.note(range) {
            warn!(
                "{} distinct visibility ranges in this run, over the bound of {RANGE_BOUND}: Bevy \
                 culls wrongly past 65535. Last: start {:?}, end {:?}",
                seen.count(),
                range.start_margin,
                range.end_margin,
            );
        }
    }
}

/// Registers the check in dev builds.
pub(super) fn plugin(app: &mut App) {
    if cfg!(feature = "dev") {
        app.add_systems(Last, watch.run_if(any_with_component::<VisibilityRange>));
    }
}
