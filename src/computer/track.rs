//! Discovery while flying: recording visits, and charting what they reveal.

use std::collections::HashSet;

use bevy::prelude::*;

use super::atlas::Atlas;
use super::board::Board;
use super::locate::whereabouts;
use super::reveal::systems_around;
use super::route::{Destination, RouteStatus, Target, remember};
use crate::screens::Screen;
use crate::space::Anchor;
use crate::universe::{StarId, Universe};
use crate::world::WorldSeed;

/// Every system the atlas reveals, charted one visited system per frame.
///
/// It is rebuilt from the atlas after a load and never saved.
#[derive(Resource, Debug, Clone, Default)]
pub struct Known {
    /// Identifiers of the known systems, in every galaxy.
    stars: HashSet<StarId>,
    /// Number of visited systems of the atlas already charted.
    done: usize,
}

impl Known {
    /// Returns true when every visited system of `atlas` is charted.
    pub fn complete(&self, atlas: &Atlas) -> bool {
        self.done == atlas.systems.len()
    }

    /// Returns the known systems in no particular order.
    pub fn stars(&self) -> impl Iterator<Item = StarId> + '_ {
        self.stars.iter().copied()
    }

    /// Charts the next visited system of `atlas`. Starts over when the atlas is a shorter one.
    pub fn absorb(&mut self, universe: &Universe, atlas: &Atlas) {
        if self.done > atlas.systems.len() {
            *self = Known::default();
        }
        if let Some(&id) = atlas.systems.get(self.done) {
            let found = systems_around(universe, id);
            self.stars.extend(found.iter().map(|star| star.id));
            self.done += 1;
        }
    }
}

/// Records the planet, system and galaxy the ship or the player is inside.
fn track(universe: Res<Universe>, anchor: Res<Anchor>, mut atlas: ResMut<Atlas>) {
    let (body, star, galaxy) = whereabouts(&universe, anchor.frame, anchor.origin);
    let inside = [
        body.map(Target::Body),
        star.map(Target::Star),
        galaxy.map(Target::Galaxy),
    ];
    for target in inside.into_iter().flatten() {
        if !atlas.visited(target) {
            atlas.visit(target);
        }
    }
}

/// Charts one more visited system while some are left.
fn chart(universe: Res<Universe>, atlas: Res<Atlas>, mut known: ResMut<Known>) {
    if !known.complete(&atlas) {
        known.absorb(&universe, &atlas);
    }
}

/// Clears the chart, the screen and the route for the world being loaded.
///
/// A world started without its save, as test runs do, gets the atlas of a new game.
fn reset(mut commands: Commands, seed: Option<Res<WorldSeed>>, atlas: Option<Res<Atlas>>) {
    commands.insert_resource(Known::default());
    commands.insert_resource(Board::default());
    commands.insert_resource(RouteStatus::default());
    if atlas.is_none() {
        let universe = Universe::new(seed.map_or(0, |seed| seed.0));
        commands.insert_resource(Atlas::new(&universe));
        commands.insert_resource(Destination::default());
    }
}

/// Run condition: a world and its atlas exist.
fn charting(
    universe: Option<Res<Universe>>,
    anchor: Option<Res<Anchor>>,
    atlas: Option<Res<Atlas>>,
) -> bool {
    universe.is_some() && anchor.is_some() && atlas.is_some()
}

/// Registers the discovery systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), reset)
        .add_systems(Update, (track, chart, remember).chain().run_if(charting));
}
