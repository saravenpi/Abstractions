//! Starting a world, new or loaded.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ActiveWorld;
use super::model::WorldEntry;
use super::store::now;
use crate::menus::Menu;
use crate::screens::Screen;
use crate::world::WorldSeed;

/// Starts worlds: inserts their resources and moves to the loading screen.
#[derive(SystemParam)]
pub struct Starter<'w, 's> {
    commands: Commands<'w, 's>,
    screen: ResMut<'w, NextState<Screen>>,
    menu: ResMut<'w, NextState<Menu>>,
}

impl Starter<'_, '_> {
    /// Makes `entry` the active world and begins loading it.
    pub fn start(&mut self, mut entry: WorldEntry) {
        entry.save.last_played = now();
        self.commands.insert_resource(WorldSeed(entry.save.seed));
        self.commands.insert_resource(ActiveWorld(entry));
        self.screen.set(Screen::Loading);
        self.menu.set(Menu::None);
    }
}
