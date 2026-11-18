//! First visits into the log of changes.
//!
//! [`crate::computer::Atlas`] keeps the planets visited, oldest first, and only ever grows. Each
//! planet it gains during play goes into [`Changes`] as a [`Change::Visited`] of the local
//! player, at the universe time of the frame. What an atlas already holds when it arrives, from
//! a save or a new game, is not a visit of this session and is not logged again. Every world,
//! new or loaded, starts on the loading screen, so the mark is cleared there: Bevy does not
//! mark a resource inserted over another as added, so the atlas itself cannot tell.

use bevy::prelude::*;

use super::{Change, Changes, PlayerId};
use crate::computer::Atlas;
use crate::screens::Screen;
use crate::space::Clock;

/// Number of planets of the atlas already logged or already held when it arrived, none until
/// the logger has seen the atlas of the world being played.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
struct Seen(Option<usize>);

/// Appends each planet the atlas gained since the last frame to the log. The first sight of
/// the atlas of a world sets the mark without logging.
fn log_visits(
    atlas: Option<Res<Atlas>>,
    clock: Option<Res<Clock>>,
    changes: Option<ResMut<Changes>>,
    mut seen: ResMut<Seen>,
) {
    let (Some(atlas), Some(clock), Some(mut changes)) = (atlas, clock, changes) else {
        return;
    };
    if let Some(logged) = seen.0 {
        for &body in atlas.bodies.iter().skip(logged) {
            changes.push(PlayerId::LOCAL, clock.seconds, Change::Visited(body));
        }
    }
    seen.set_if_neq(Seen(Some(atlas.bodies.len())));
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

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::computer::Target;
    use crate::universe::{BodyId, Universe};

    /// Returns an app on the title screen with the logger, a new game's atlas and the clock at
    /// `seconds`.
    fn app(seconds: f64) -> App {
        let mut app = App::new();
        app.add_plugins((StatesPlugin, plugin))
            .init_state::<Screen>()
            .init_resource::<Changes>()
            .insert_resource(Atlas::new(&Universe::new(9)))
            .insert_resource(Clock { seconds });
        app.update();
        app
    }

    /// The atlas of a new game logs nothing; a first visit appends one entry at the universe
    /// time, a second visit of the same planet none.
    #[test]
    fn a_first_visit_appends_one_entry() {
        let mut app = app(1234.5);
        assert!(app.world().resource::<Changes>().entries.is_empty());
        let home = Universe::new(9).home();
        let other = BodyId {
            index: home.index + 1,
            ..home
        };
        for _ in 0..2 {
            let mut atlas = app.world_mut().resource_mut::<Atlas>();
            atlas.visit(Target::Body(other));
            app.update();
        }
        let entries = &app.world().resource::<Changes>().entries;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].change, Change::Visited(other));
        assert_eq!(
            (entries[0].player, entries[0].time),
            (PlayerId::LOCAL, 1234.5)
        );
    }

    /// A world loaded in place of another brings an atlas whose planets are not logged again.
    #[test]
    fn a_loaded_atlas_is_not_logged() {
        let mut app = app(0.0);
        let mut atlas = Atlas::new(&Universe::new(9));
        let home = atlas.bodies[0];
        atlas.visit(Target::Body(BodyId { moon: 1, ..home }));
        app.insert_resource(atlas);
        let world = app.world_mut();
        world
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Loading);
        app.update();
        app.update();
        assert!(app.world().resource::<Changes>().entries.is_empty());
    }
}
