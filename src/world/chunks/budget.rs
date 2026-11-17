//! The integration budget: how many milliseconds of a frame the main thread may spend turning
//! streamed results into entities.

use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::reach::Reach;
use crate::screens::{Screen, WarmUp};
use crate::settings::{Persistent, Settings};

/// Milliseconds one frame of play may spend on each kind of streamed work.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shares {
    /// Finished detail chunks turned into entities.
    pub chunks: f32,
    /// Finished coarse nodes turned into entities.
    pub nodes: f32,
    /// Rebuilt terrain meshes and ground cover swapped in.
    pub swaps: f32,
}

/// The budget of a frame of play.
pub const PLAY: Shares = Shares {
    chunks: 1.5,
    nodes: 1.0,
    swaps: 0.5,
};

/// The budget of a frame behind the loading or the boot screen, where a long frame shows nothing.
pub const COVERED: Shares = Shares {
    chunks: 14.0,
    nodes: 8.0,
    swaps: 4.0,
};

/// The budgets the streamer works with this frame.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Budget(pub Shares);

impl Default for Budget {
    /// Returns the budget of a covered frame: a world starts behind the loading screen.
    fn default() -> Self {
        Self(COVERED)
    }
}

/// One share of the budget being spent.
pub struct Spend {
    start: Instant,
    limit: Duration,
    items: usize,
}

impl Spend {
    /// Starts spending `millis` milliseconds.
    pub fn of(millis: f32) -> Self {
        Self {
            start: Instant::now(),
            limit: Duration::from_secs_f32(millis.max(0.0) / 1000.0),
            items: 0,
        }
    }

    /// Returns true while there is time left for one more item, and counts it. The first item
    /// always goes through, so a budget smaller than one item still makes progress.
    pub fn more(&mut self) -> bool {
        let room = self.items == 0 || self.start.elapsed() < self.limit;
        self.items += usize::from(room);
        room
    }
}

/// Picks the budget of the frame: generous while the loading or the boot screen covers the world,
/// tight during play.
pub fn set_budget(
    mut budget: ResMut<Budget>,
    screen: Res<State<Screen>>,
    warm: Option<Res<State<WarmUp>>>,
) {
    let hidden = warm.is_some_and(|warm| *warm.get() == WarmUp::Hidden);
    let wanted = Budget(match *screen.get() != Screen::Gameplay || hidden {
        true => COVERED,
        false => PLAY,
    });
    budget.set_if_neq(wanted);
}

/// Keeps the [`Reach`] resource in step with the render distance setting.
pub fn follow_setting(settings: Option<Res<Persistent<Settings>>>, mut reach: ResMut<Reach>) {
    if let Some(settings) = settings.filter(|s| s.is_changed()) {
        reach.set_if_neq(Reach::of(Some(settings.graphics.render_distance)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A spent budget still lets its first item through, and then stops.
    #[test]
    fn budget_always_makes_progress() {
        let mut none = Spend::of(0.0);
        assert!(none.more());
        assert!(!none.more());
        let mut plenty = Spend::of(1000.0);
        assert!((0..50).all(|_| plenty.more()));
    }
}
