//! First visits into the log of changes.
//!
//! [`crate::computer::Atlas`] keeps the planets visited, oldest first, and only ever grows. Each
//! planet it gains during play goes into [`Changes`] as a [`Change::Visited`] of the local
//! player, at the universe time of the frame. What an atlas already holds when it arrives, from
//! a save or a new game, is not a visit of this session and is not logged again. Bevy does not
//! mark a resource inserted over another as added, so the atlas itself cannot tell: the mark
//! is kept for one world, named by its save and its seed, and the atlas of any other world is
//! new. A world starts from a system of `Update`, one frame before the loading screen, so
//! clearing the mark on that screen alone would come one comparison too late.

use bevy::prelude::*;

use super::{Change, Changes, PlayerId};
use crate::computer::Atlas;
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::space::Clock;

/// How far the logger has read the atlas of one world.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
struct Seen {
    /// The world the mark belongs to: the id of its save and its seed. Nothing without a save.
    world: Option<(String, u64)>,
    /// Number of planets of the atlas already logged or already held when it arrived, none
    /// until the logger has seen the atlas of that world.
    logged: Option<usize>,
}

impl Seen {
    /// Returns true when the mark belongs to the world named by `world`.
    fn of(&self, world: Option<(&str, u64)>) -> bool {
        let own = self.world.as_ref().map(|(id, seed)| (id.as_str(), *seed));
        own == world
    }
}

/// Appends each planet the atlas gained since the last frame to the log. The first sight of
/// the atlas of a world sets the mark without logging, and so does the atlas of another world
/// than the one the mark was set in.
fn log_visits(
    atlas: Option<Res<Atlas>>,
    clock: Option<Res<Clock>>,
    changes: Option<ResMut<Changes>>,
    active: Option<Res<ActiveWorld>>,
    mut seen: ResMut<Seen>,
) {
    let (Some(atlas), Some(clock), Some(mut changes)) = (atlas, clock, changes) else {
        return;
    };
    let world = active.as_deref().map(|active| &active.0);
    let world = world.map(|entry| (entry.id.as_str(), entry.save.seed));
    let same = seen.of(world);
    if let Some(logged) = seen.logged.filter(|_| same) {
        for &body in atlas.bodies.iter().skip(logged) {
            changes.push(PlayerId::LOCAL, clock.seconds, Change::Visited(body));
        }
    }
    if !same {
        seen.world = world.map(|(id, seed)| (id.to_string(), seed));
    }
    if seen.logged != Some(atlas.bodies.len()) {
        seen.logged = Some(atlas.bodies.len());
    }
}

/// Forgets the mark: the world being loaded brings its own atlas and log.
fn forget(mut seen: ResMut<Seen>) {
    *seen = Seen::default();
}

/// Registers the log of first visits, after every system of the frame that visits.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Seen>()
        .add_systems(OnEnter(Screen::Loading), forget)
        .add_systems(PostUpdate, log_visits);
}
