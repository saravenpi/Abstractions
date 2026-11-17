//! Worlds on the rig: started the way the menus start them, saved the way the pause menu saves.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::Game;
use crate::menus::Menu;
use crate::saves::{SaveDir, Starter, WorldEntry, Writer, list, new_entry};
use crate::screens::Screen;

impl Game {
    /// Starts `entry`, or a new world named `name` without one, and waits until it is played.
    pub fn start(&mut self, name: &str, entry: Option<WorldEntry>) {
        let dir = self.app.world().resource::<SaveDir>().0.clone();
        let entry = entry.unwrap_or_else(|| new_entry(&dir, name, 7));
        let begun = self
            .app
            .world_mut()
            .run_system_once(move |mut starter: Starter| starter.start(entry.clone()));
        begun.expect("the world starts");
        self.wait("the world to load", |game| {
            *game.app.world().resource::<State<Screen>>().get() == Screen::Gameplay
        });
    }

    /// Waits until no world file is being written or waits to be.
    fn written(&mut self) {
        self.step(2);
        self.wait("the save file", |game| {
            !game.app.world().resource::<Writer>().busy()
        });
    }

    /// Saves by opening the pause menu, waits for the file, goes back to the title screen and
    /// returns the world as the file holds it.
    pub fn save_and_quit(&mut self) -> WorldEntry {
        let dir = self.app.world().resource::<SaveDir>().0.clone();
        let world = self.app.world_mut();
        world.resource_mut::<NextState<Menu>>().set(Menu::Pause);
        self.written();
        let world = self.app.world_mut();
        world.resource_mut::<NextState<Screen>>().set(Screen::Title);
        world.resource_mut::<NextState<Menu>>().set(Menu::None);
        self.written();
        list(&dir).remove(0)
    }
}
