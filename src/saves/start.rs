//! Starting a world, new or loaded.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ActiveWorld;
use super::model::WorldEntry;
use super::store::now;
use crate::computer::{Atlas, Destination};
use crate::menus::Menu;
use crate::screens::Screen;
use crate::space::Clock;
use crate::universe::Universe;
use crate::world::WorldSeed;

/// Run condition: the world being loaded has never been played on a planet, so it starts as a
/// new game. True for a new world, for a file from before v0.6, and without any world file.
pub fn unplayed(active: Option<Res<ActiveWorld>>) -> bool {
    active.is_none_or(|active| active.0.save.place.is_none())
}

/// Starts worlds: inserts their resources and moves to the loading screen.
#[derive(SystemParam)]
pub struct Starter<'w, 's> {
    commands: Commands<'w, 's>,
    screen: ResMut<'w, NextState<Screen>>,
    menu: ResMut<'w, NextState<Menu>>,
}

impl Starter<'_, '_> {
    /// Makes `entry` the active world and begins loading it.
    ///
    /// The universe clock, the atlas, the destination and the log of changes are restored here. The planet and the
    /// spot on it are read from the save by the world, through [`super::WorldSave::body`] and
    /// [`super::WorldSave::spawn`]. A file from before v0.6 starts at the home planet spawn.
    pub fn start(&mut self, mut entry: WorldEntry) {
        entry.save.last_played = now();
        entry.save.upgrade();
        let universe = Universe::new(entry.save.seed);
        let atlas = entry.save.atlas.clone();
        let atlas = atlas.unwrap_or_else(|| Atlas::new(&universe));
        self.commands
            .insert_resource(Destination(atlas.destination));
        self.commands.insert_resource(atlas);
        self.commands.insert_resource(entry.save.changes.clone());
        self.commands.insert_resource(Clock {
            seconds: entry.save.clock_seconds,
        });
        self.commands.insert_resource(WorldSeed(entry.save.seed));
        self.commands.insert_resource(ActiveWorld(entry));
        self.screen.set(Screen::Loading);
        self.menu.set(Menu::None);
    }
}
